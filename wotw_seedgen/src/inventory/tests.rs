use crate::{inventory::Inventory, orbs::Orbs};
use smallvec::{smallvec, SmallVec};
use wotw_seedgen_data::{Shard, Skill};
use wotw_seedgen_logic_language::output::RefillValue;
use wotw_seedgen_settings::{Difficulty, WorldSettings};

// TODO these tests look like they belong into Inventory now
#[test]
fn weapon_preference() {
    let mut settings = WorldSettings::default();
    let mut inventory = Inventory::default();
    assert_eq!(
        inventory.progression_weapons::<false>(&settings),
        SmallVec::from_buf([
            Skill::Sword,
            Skill::Hammer,
            Skill::Bow,
            Skill::Grenade,
            Skill::Shuriken,
            Skill::Blaze,
            Skill::Flash,
            Skill::Spear,
        ])
    );
    inventory.skills.insert(Skill::Shuriken);
    assert_eq!(
        inventory.progression_weapons::<false>(&settings),
        SmallVec::from_buf([
            Skill::Sword,
            Skill::Hammer,
            Skill::Bow,
            Skill::Grenade,
            Skill::Shuriken,
        ])
    );
    settings.difficulty = Difficulty::Unsafe;
    assert_eq!(
        inventory.progression_weapons::<false>(&settings),
        SmallVec::from_buf([
            Skill::Sword,
            Skill::Hammer,
            Skill::Bow,
            Skill::Grenade,
            Skill::Shuriken,
        ])
    );
}

#[test]
fn max_energy() {
    let mut inventory = Inventory::default();
    assert_eq!(inventory.max_energy(Difficulty::Moki), 0.0);
    inventory.energy += 5.;
    inventory.shards.insert(Shard::Energy);
    assert_eq!(inventory.max_energy(Difficulty::Moki), 5.0);
    assert_eq!(inventory.max_energy(Difficulty::Gorlek), 6.0);
}

#[test]
fn refill_orbs() {
    let mut inventory = Inventory::spawn();

    let expected = [
        30., 35., 40., 40., 40., 40., 40., 40., 40., 40., 40., 40., 40., 40., 40., 40., 40., 40.,
        40., 40., 40., 41., 42., 44., 45., 47., 48., 50., 52., 53., 55., 56., 58., 59., 61., 62.,
        64., 65., 66., 68., 69.,
    ];
    for health in expected {
        assert_eq!(inventory.checkpoint_orbs(Difficulty::Moki).health, health);
        inventory.health += 5;
    }

    inventory.clear();

    let expected = [
        0., 0., 0., 0., 1., 1., 1., 1., 1., 2., 2., 2., 2., 2., 2., 2., 3., 3., 3., 3., 3., 4., 4.,
        4., 4., 4., 4., 4., 5., 5., 5., 5., 5., 6., 6., 6., 6., 6., 6., 6., 7., 7., 7., 7., 7., 8.,
        8.,
    ];
    for drops in expected {
        assert_eq!(inventory.health_plant_drops(Difficulty::Moki), drops);
        inventory.health += 5;
    }

    inventory.clear();

    inventory.shards.insert(Shard::Energy);
    inventory.shards.insert(Shard::Vitality);

    assert_eq!(
        inventory.checkpoint_orbs(Difficulty::Gorlek),
        Orbs {
            energy: 1.0,
            health: 0.0
        }
    );

    inventory.health += 35;

    assert_eq!(
        inventory.checkpoint_orbs(Difficulty::Gorlek),
        Orbs {
            health: 35.0,
            energy: 1.0
        }
    );

    inventory.health += 105;

    assert_eq!(
        inventory.checkpoint_orbs(Difficulty::Gorlek),
        Orbs {
            health: 45.0,
            energy: 1.0
        }
    );

    inventory = Inventory::spawn();

    let mut orb_variants = smallvec![Orbs::default()];
    inventory.refill(RefillValue::Full, &mut orb_variants, Difficulty::Moki);
    assert_eq!(&orb_variants[..], &[inventory.max_orbs(Difficulty::Moki)]);
}

#[test]
fn destroy_cost() {
    let mut world_settings = WorldSettings::default();
    let mut inventory = Inventory::default();
    assert_eq!(
        inventory.destroy_cost::<false>(10.0, false, &world_settings),
        None
    );
    inventory.skills.insert(Skill::Spear);
    assert_eq!(
        inventory.destroy_cost::<false>(10.0, true, &world_settings),
        Some(4.0)
    );
    assert_eq!(
        inventory.destroy_cost::<false>(0.0, false, &world_settings),
        Some(0.0)
    );
    inventory.skills.insert(Skill::Bow);
    assert_eq!(
        inventory.destroy_cost::<false>(10.0, false, &world_settings),
        Some(1.5)
    );
    world_settings.difficulty = Difficulty::Unsafe;
    inventory.skills.insert(Skill::GladesAncestralLight);
    inventory.skills.insert(Skill::InkwaterAncestralLight);
    inventory.shards.insert(Shard::Wingclip);
    inventory.shard_slots += 1;
    inventory.skills.remove(&Skill::Bow);
    assert_eq!(
        inventory.destroy_cost::<false>(1.0, false, &world_settings),
        Some(2.0)
    );
    inventory.skills.insert(Skill::Bow);
    assert_eq!(
        inventory.destroy_cost::<false>(10.0, true, &world_settings),
        Some(0.25)
    );
    inventory.clear();
    inventory.skills.insert(Skill::Grenade);
    inventory.skills.insert(Skill::Shuriken);
    assert_eq!(
        inventory.destroy_cost::<false>(20.0, false, &world_settings),
        Some(1.5)
    );
    assert_eq!(
        inventory.destroy_cost::<false>(24.0, false, &world_settings),
        Some(1.5)
    );
    assert_eq!(
        inventory.destroy_cost::<false>(34.0, false, &world_settings),
        Some(2.0)
    );
}
