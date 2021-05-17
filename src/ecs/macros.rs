macro_rules! all_tuples {
    ($macro: ident) => {
        $macro!(A);
        $macro!(A, B);
        $macro!(A, B, C);
        $macro!(A, B, C, D);
        $macro!(A, B, C, D, E);
        $macro!(A, B, C, D, E, F);
        $macro!(A, B, C, D, E, F, G);
        $macro!(A, B, C, D, E, F, G, H);
        $macro!(A, B, C, D, E, F, G, H, I);
        $macro!(A, B, C, D, E, F, G, H, I, J);
        $macro!(A, B, C, D, E, F, G, H, I, J, K);
        $macro!(A, B, C, D, E, F, G, H, I, J, K, L);
    };
}
