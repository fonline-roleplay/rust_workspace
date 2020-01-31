pub use fo_defines::DamageType;

pub trait Engine: Sized {
    //TODO: generate MAX_DETERIORATION from headers?
    const MAX_DETERIORATION: i32;

    type Item: ItemLike<Self>;
    type ItemProto: ItemProtoLike<Self>;
    type Critter: CritterLike<Self>;
    type Param: Copy;
}

pub trait CritterLike<E: Engine> {
    fn param(&self, param: E::Param) -> i32;
    fn armor(&self) -> Option<&E::Item>;
}

pub trait ItemLike<E: Engine> {
    fn proto(&self) -> &E::ItemProto;
    fn deterioration(&self) -> i32;
    fn deterioration_proc(&self) -> i32 {
        let proc = self.deterioration() * 100 / E::MAX_DETERIORATION;
        proc.min(100).max(0)
    }
    fn durability_proc(&self) -> i32 {
        100 - self.deterioration_proc()
    }
    fn resist_proc(&self, damage: DamageType) -> i32 {
        self.proto().resist(damage) * self.durability_proc() / 100
    }
    fn absorb_proc(&self, damage: DamageType) -> i32 {
        self.proto().absorb(damage) * self.durability_proc() / 100
    }
}

pub trait ItemProtoLike<E: Engine> {
    fn resist(&self, damage: DamageType) -> i32;
    fn absorb(&self, damage: DamageType) -> i32;
    fn armor_class(&self) -> i32;
}

#[macro_export]
macro_rules! impl_engine ((
    impl Engine for $engine:path {
        type Item = $item:path;
        type ItemProto = $item_proto:path;
        type Critter = $critter:path;
        type Param = $param:path;
    }
) => {
    impl $crate::Engine for $engine {
        const MAX_DETERIORATION: i32 = 10000;
        type Item = $item;
        type ItemProto = $item_proto;
        type Critter = $critter;
        type Param = $param;
    }
    impl $crate::CritterLike<$engine> for $critter {
        fn param(&self, param: $param) -> i32 {
            self.Params[param as usize]
        }
        fn armor(&self) -> Option<&$item> {
            let item = unsafe { self.ItemSlotArmor.as_ref() };
            item.filter(|item| !item.IsNotValid && !item.Proto.is_null())
        }
    }
    impl $crate::ItemLike<$engine> for $item {
        fn proto(&self) -> &$item_proto {
            unsafe { self.Proto.as_ref().expect("ItemProto") }
        }
        fn deterioration(&self) -> i32 {
            self.Data.Deterioration as i32
        }
    }
    impl $crate::ItemProtoLike<$engine> for $item_proto {
        fn resist(&self, damage: $crate::DamageType) -> i32 {
            use $crate::DamageType::*;
            match damage {
                Uncalled => 0,
                Normal => self.Armor_DRNormal,
                Laser => self.Armor_DRLaser,
                Fire => self.Armor_DRFire,
                Plasma => self.Armor_DRPlasma,
                Electric => self.Armor_DRElectr,
                Emp => self.Armor_DREmp,
                Explosion => self.Armor_DRExplode,
                Unknown => 0,
            }
        }
        fn absorb(&self, damage: $crate::DamageType) -> i32 {
            use $crate::DamageType::*;
            match damage {
                Uncalled => 0,
                Normal => self.Armor_DTNormal,
                Laser => self.Armor_DTLaser,
                Fire => self.Armor_DTFire,
                Plasma => self.Armor_DTPlasma,
                Electric => self.Armor_DTElectr,
                Emp => self.Armor_DTEmp,
                Explosion => self.Armor_DTExplode,
                Unknown => 0,
            }
        }
        fn armor_class(&self) -> i32 {
            self.Armor_AC
        }
    }
});

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
