#[macro_export]
macro_rules! gen_mut{
    ($m:ty => $code:expr) => {
        match <$m as $crate::Mutability>::mutability() {
            $crate::MutabilityEnum::Shared(proof) => {
                macro_rules! into_gen {
                    () => (|genref| $crate::GenRef::gen_from_shared(genref, proof));
                    (&gen $genref:expr) => ($crate::GenRef::gen_from_shared(& $genref, proof));
                    ($genref:expr) => ($crate::GenRef::gen_from_shared($genref, proof))
                }
                macro_rules! from_gen {
                    () => (|genref| $crate::GenRef::gen_into_shared(genref, proof));
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
                    () => (|genref| $crate::GenRef::gen_from_mut(genref, proof));
                    (&gen $genref:expr) => ($crate::GenRef::gen_from_mut(&mut $genref, proof));
                    ($genref:expr) => ($crate::GenRef::gen_from_mut($genref, proof))
                }
                macro_rules! from_gen {
                    () => (|genref| $crate::GenRef::gen_into_mut(genref, proof));
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

#[macro_export]
macro_rules! field {
    (&gen $genref:tt $(. $field:tt)+) => {
        $crate::GenRef::map($genref, |r| &mut r $(. $field)+, |r| & r $(. $field)+)
    };
}