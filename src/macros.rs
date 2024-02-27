#[macro_export]
macro_rules! gen_mut{
    ($m:ty => $code:expr) => {
        match <$m as $crate::Mutability>::mutability() {
            $crate::MutabilityEnum::Shared(proof) => {
                macro_rules! into_gen {
                    () => (|gen_ref| $crate::GenRef::gen_from_shared(gen_ref, proof));
                    (&gen $gen_ref:expr) => ($crate::GenRef::gen_from_shared(& $gen_ref, proof));
                    ($gen_ref:expr) => ($crate::GenRef::gen_from_shared($gen_ref, proof))
                }
                macro_rules! from_gen {
                    () => (|gen_ref| $crate::GenRef::gen_into_shared(gen_ref, proof));
                    ($reference:expr) => ($crate::GenRef::gen_into_shared($reference, proof))
                }
                #[allow(unused_macros)]
                macro_rules! switch_mut_shared {
                    ($mutable:expr, $shared:expr) => ($shared)
                }
                $code
            },
            $crate::MutabilityEnum::Mutable(proof) => {
                macro_rules! into_gen {
                    () => (|gen_ref| $crate::GenRef::gen_from_mut(gen_ref, proof));
                    (&gen $gen_ref:expr) => ($crate::GenRef::gen_from_mut(&mut $gen_ref, proof));
                    ($gen_ref:expr) => ($crate::GenRef::gen_from_mut($gen_ref, proof))
                }
                macro_rules! from_gen {
                    () => (|gen_ref| $crate::GenRef::gen_into_mut(gen_ref, proof));
                    ($reference:expr) => ($crate::GenRef::gen_into_mut($reference, proof))
                }
                #[allow(unused_macros)]
                macro_rules! switch_mut_shared {
                    ($mutable:expr, $shared:expr) => ($mutable)
                }
                $code
            },
        }
    }
}