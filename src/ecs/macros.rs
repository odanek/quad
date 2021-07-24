macro_rules! all_tuples {
    ($macro: ident) => {
        $macro!();
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

macro_rules! all_pair_tuples {
    ($macro: ident) => {
        $macro!();
        $macro!((A1, A2));
        $macro!((A1, A2), (B1, B2));
        $macro!((A1, A2), (B1, B2), (C1, C2));
        $macro!((A1, A2), (B1, B2), (C1, C2), (D1, D2));
        $macro!((A1, A2), (B1, B2), (C1, C2), (D1, D2), (E1, E2));
        $macro!((A1, A2), (B1, B2), (C1, C2), (D1, D2), (E1, E2), (F1, F2));
        $macro!(
            (A1, A2),
            (B1, B2),
            (C1, C2),
            (D1, D2),
            (E1, E2),
            (F1, F2),
            (G1, G2)
        );
        $macro!(
            (A1, A2),
            (B1, B2),
            (C1, C2),
            (D1, D2),
            (E1, E2),
            (F1, F2),
            (G1, G2),
            (H1, H2)
        );
        $macro!(
            (A1, A2),
            (B1, B2),
            (C1, C2),
            (D1, D2),
            (E1, E2),
            (F1, F2),
            (G1, G2),
            (H1, H2),
            (I1, I2)
        );
        $macro!(
            (A1, A2),
            (B1, B2),
            (C1, C2),
            (D1, D2),
            (E1, E2),
            (F1, F2),
            (G1, G2),
            (H1, H2),
            (I1, I2),
            (J1, J2)
        );
        $macro!(
            (A1, A2),
            (B1, B2),
            (C1, C2),
            (D1, D2),
            (E1, E2),
            (F1, F2),
            (G1, G2),
            (H1, H2),
            (I1, I2),
            (J1, J2),
            (K1, K2)
        );
    };
}
