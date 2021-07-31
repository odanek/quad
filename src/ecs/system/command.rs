use std::marker::PhantomData;

use crate::ecs::{Entities, Entity, World, component::{Component, bundle::Bundle}};

struct CommandMeta {
    offset: usize,
    func: unsafe fn(value: *mut u8, world: &mut World),
}

#[derive(Default)]
pub struct CommandQueue {
    bytes: Vec<u8>,
    metas: Vec<CommandMeta>,
}

unsafe impl Send for CommandQueue {}

unsafe impl Sync for CommandQueue {}

// TODO: Consider writing this in safe Rust. Does quad really need this optimization?
impl CommandQueue {
    /// Push a [`Command`] onto the queue.
    #[inline]
    pub fn push<C>(&mut self, command: C)
    where
        C: Command,
    {
        unsafe fn write_command<T: Command>(command: *mut u8, world: &mut World) {
            let command = command.cast::<T>().read_unaligned();
            command.write(world);
        }

        let size = std::mem::size_of::<C>();
        let old_len = self.bytes.len();

        self.metas.push(CommandMeta {
            offset: old_len,
            func: write_command::<C>,
        });

        if size > 0 {
            self.bytes.reserve(size);

            unsafe {
                std::ptr::copy_nonoverlapping(
                    &command as *const C as *const u8,
                    self.bytes.as_mut_ptr().add(old_len),
                    size,
                );
                self.bytes.set_len(old_len + size);
            }
        }

        std::mem::forget(command);
    }

    #[inline]
    pub fn apply(&mut self, world: &mut World) {
        world.flush();

        unsafe { self.bytes.set_len(0) };

        let byte_ptr = if self.bytes.as_mut_ptr().is_null() {
            unsafe { std::ptr::NonNull::dangling().as_mut() }
        } else {
            self.bytes.as_mut_ptr()
        };

        for meta in self.metas.drain(..) {
            unsafe {
                (meta.func)(byte_ptr.add(meta.offset), world);
            }
        }
    }
}

pub trait Command: Send + Sync + 'static {
    fn write(self, world: &mut World);
}

pub struct Commands<'a> {
    queue: &'a mut CommandQueue,
    entities: &'a Entities,
}

