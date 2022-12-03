use std::collections::{HashMap, HashSet};
use std::process::exit;

use log::{error, info};
use queue::Queue;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use albw::Item;

use crate::{convert, LocationInfo, Seed, Settings};
use crate::check::Check;
use crate::FillerItem;
use crate::FillerItem::*;
use crate::location::Location;
use crate::location_node::LocationNode;
use crate::logic_mode::LogicMode::*;
use crate::progress::Progress;
use crate::world::build_world_graph;

/// Filler Algorithm
pub fn fill_stuff(settings: &Settings, seed: Seed) -> Vec<(LocationInfo, Item)> {
    info!("Seed:                           {}", seed);
    //info!("Hash:                           {}", settings.hash().0);
    info!("Logic:                          {}", match settings.logic.mode {
        Normal => "Normal",
        Hard => "Hard",
        GlitchBasic => "Glitched (Basic)",
        GlitchAdvanced => "Glitched (Advanced)",
        GlitchHell => "Glitched (Hell) - Did you really mean to choose this?",
        NoLogic => "No Logic",
    });
    info!("Dungeon Prizes:                 {}", if settings.logic.randomize_dungeon_prizes {"Randomized"} else {"Not Randomized"});
    info!("Maiamai:                        {}", if settings.logic.maiamai_madness {"Shuffled"} else {"Not Shuffled"});
    info!("Weather Vanes:                  {}", if settings.logic.vanes_activated {"All Activated"} else {"Normal"});
    info!("Super Items:                    {}", if settings.logic.super_items {"Included"} else {"Not Included"});
    info!("Trials:                         {}", if settings.logic.skip_trials {"Skipped"} else {"Normal"});
    info!("Dark Rooms:                     {}", if settings.logic.lampless {"Lamp Not Required"} else {"Lamp Required"});
    info!("Swords:                         {}\n", if settings.logic.swordless_mode {"Swordless Mode - NO SWORDS"} else {"Normal"});

    let mut rng = StdRng::seed_from_u64(seed as u64);

    let mut world_graph = build_world_graph();
    let mut check_map = prefill_check_map(&mut world_graph);

    let (
        mut progression_pool,
        mut trash_pool
    ) = get_item_pools(settings, &mut rng);

    verify_all_locations_accessible(&mut world_graph, &progression_pool, settings);

    handle_exclusions(&mut check_map, settings, &mut rng, &mut trash_pool);

    preplace_items(&mut check_map, settings, &mut rng, &mut progression_pool, &mut trash_pool);

    assumed_fill(&mut world_graph, &mut rng, &mut progression_pool, &mut check_map, settings);

    fill_trash(&mut check_map, &mut rng, &trash_pool);

    map_to_result(world_graph, check_map)
}

/// Place static items ahead of the randomly filled ones
fn preplace_items<'a>(check_map: &mut HashMap<&'a str, Option<FillerItem>>,
                      settings: &'a Settings,
                      rng: &mut StdRng,
                      progression: &mut Vec<FillerItem>,
                      trash: &mut Vec<FillerItem>) {

    // Place un-randomized items
    place_static(check_map, progression, LetterInABottle, "Shore");
    place_static(check_map, progression, RupeeSilver39, "Cucco Dungeon");
    place_static(check_map, progression, RupeeSilver40, "[TR] (1F) Under Center");
    place_static(check_map, progression, RupeeGold09, "[TR] (B1) Under Center");
    place_static(check_map, progression, RupeeGold10, "[PD] (2F) South Hidden Room");

    // Vanilla Dungeon Prizes
    if !settings.logic.randomize_dungeon_prizes {
        place_static(check_map, progression, PendantOfCourage, "Eastern Palace Prize");
        place_static(check_map, progression, PendantOfWisdom, "House of Gales Prize");
        place_static(check_map, progression, PendantOfPower, "Tower of Hera Prize");
        place_static(check_map, progression, SageGulley, "Dark Palace Prize");
        place_static(check_map, progression, SageOren, "Swamp Palace Prize");
        place_static(check_map, progression, SageSeres, "Skull Woods Prize");
        place_static(check_map, progression, SageOsfala, "Thieves' Hideout Prize");
        place_static(check_map, progression, SageImpa, "Turtle Rock Prize");
        place_static(check_map, progression, SageIrene, "Desert Palace Prize");
        place_static(check_map, progression, SageRosso, "Ice Ruins Prize");
    }

    let mut shop_positions: Vec<&str> = Vec::new();
    let mut bow_light_positions: Vec<&str> = Vec::from(["Zelda"]);
    let mut maiamai_positions: Vec<&str> = Vec::new();

    for (check_name, item) in check_map.clone() {
        if check_name.starts_with("[LC]") && item.is_none() {
            let _ = &bow_light_positions.push(check_name);
        } else if check_name.starts_with("Ravio") && !check_name.contains("6") {
            let _ = &shop_positions.push(check_name);
        } else if check_name.starts_with("[Mai]") {
            let _ = &maiamai_positions.push(check_name);
        }
    }

    if settings.logic.bow_of_light_in_castle {
        check_map.insert(bow_light_positions.remove(rng.gen_range(0..bow_light_positions.len())), Some(BowOfLight));
        progression.retain(|x| *x != BowOfLight);
    }

    // Assures a weapon will be available in Ravio's Shop
    if settings.logic.assured_weapon {
        let mut weapons = Vec::from([
            Bow01, Bombs01, FireRod01, IceRod01, Hammer01
        ]);

        if !settings.logic.swordless_mode {
            weapons.append(&mut Vec::from([Sword01, Sword02, Sword03, Sword04]));
        }

        match settings.logic.mode {
            Normal => {}
            _ => {
                weapons.append(&mut Vec::from([Lamp01, Net01]));
            }
        }

        let weapon = *weapons.get(rng.gen_range(0..weapons.len())).unwrap();

        check_map.insert(shop_positions.remove(rng.gen_range(0..shop_positions.len())), Some(weapon));
        progression.retain(|x| *x != weapon);
    }

    if settings.logic.bell_in_shop {
        check_map.insert(shop_positions.remove(rng.gen_range(0..shop_positions.len())), Some(Bell));
        progression.retain(|x| *x != Bell);
    }

    if settings.logic.pouch_in_shop {
        check_map.insert(shop_positions.remove(rng.gen_range(0..shop_positions.len())), Some(Pouch));
        progression.retain(|x| *x != Pouch);
    }

    if settings.logic.boots_in_shop {
        check_map.insert(shop_positions.remove(rng.gen_range(0..shop_positions.len())), Some(PegasusBoots));
        progression.retain(|x| *x != PegasusBoots);
    }

    // Exclude Minigames
    if settings.logic.minigames_excluded {
        exclude("Cucco Ranch", rng, check_map, trash);
        exclude("Hyrule Hotfoot", rng, check_map, trash);
        exclude("Rupee Rush (Hyrule)", rng, check_map, trash);
        exclude("Rupee Rush (Lorule)", rng, check_map, trash);
        exclude("Octoball Derby", rng, check_map, trash);
        exclude("Treacherous Tower (Intermediate)", rng, check_map, trash);

        // For Maiamai Madness, also turn the rupee rush maiamai into random trash
        if settings.logic.maiamai_madness {
            exclude("[Mai] Hyrule Rupee Rush Wall", rng, check_map, trash);
            exclude("[Mai] Lorule Rupee Rush Wall", rng, check_map, trash);
        }
    }

    // For non-Maiamai Madness seeds, default them to Maiamai
    if !settings.logic.maiamai_madness {
        let mut maiamai_items = maiamai_pool();
        for check_name in maiamai_positions {
            // FIXME Inefficient to add Maiamai to progression pool, shuffle, then remove them
            place_static(check_map, progression, maiamai_items.remove(0), check_name);
        }
    }
}

// Statically place an item in a give location, then remove it from the item pool provided
fn place_static<'a>(check_map: &mut HashMap<&'a str, Option<FillerItem>>, pool: &mut Vec<FillerItem>, item: FillerItem, check_name: &'a str) {
    check_map.insert(check_name, Some(item));
    pool.retain(|x| *x != item);
}

// Exclude a location by placing a random trash item there
fn exclude(check_name: &'static str, rng: &mut StdRng, check_map: &mut HashMap<&str, Option<FillerItem>>, trash: &mut Vec<FillerItem>) {
    check_map.insert(check_name, Some(trash.remove(rng.gen_range(0..trash.len()))));
}

fn handle_exclusions<'a>(check_map: &mut HashMap<&'a str, Option<FillerItem>>,
                         settings: &'a Settings,
                         rng: &mut StdRng,
                         trash_pool: &mut Vec<FillerItem>) {
    let opt = settings.exclusions.0.get("exclusions");
    if opt.is_none() {
        return;
    }

    let exclusions = opt.unwrap();

    for exclusion in exclusions {
        if check_map.contains_key(&exclusion.as_str()) {
            check_map.insert(&exclusion.as_str(), Some(trash_pool.remove(rng.gen_range(0..trash_pool.len()))));
        } else {
            error!("Cannot exclude \"{}\", no matching check found with that name.", &exclusion.as_str());
            error!("Consult a spoiler log for a list of valid check names.");
            exit(1);
        }
    }
}

/// Super dirty mapping I hate it
fn map_to_result(world_graph: HashMap<Location, LocationNode>, check_map: HashMap<&str, Option<FillerItem>>) -> Vec<(LocationInfo, Item)> {
    let mut result: Vec<(LocationInfo, Item)> = Vec::new();
    for (_, location_node) in world_graph {
        for check in location_node.get_checks() {
            if check.get_location_info().is_some() {
                result.push((
                    check.get_location_info().unwrap(),
                    convert(check_map.get(check.get_name()).unwrap().unwrap()).unwrap()));
            }
        }
    }
    result
}

fn maiamai_pool() -> Vec<FillerItem> {
    vec![
        Maiamai001,
        Maiamai002,
        Maiamai003,
        Maiamai004,
        Maiamai005,
        Maiamai006,
        Maiamai007,
        Maiamai008,
        Maiamai009,
        Maiamai010,
        Maiamai011,
        Maiamai012,
        Maiamai013,
        Maiamai014,
        Maiamai015,
        Maiamai016,
        Maiamai017,
        Maiamai018,
        Maiamai019,
        Maiamai020,
        Maiamai021,
        Maiamai022,
        Maiamai023,
        Maiamai024,
        Maiamai025,
        Maiamai026,
        Maiamai027,
        Maiamai028,
        Maiamai029,
        Maiamai030,
        Maiamai031,
        Maiamai032,
        Maiamai033,
        Maiamai034,
        Maiamai035,
        Maiamai036,
        Maiamai037,
        Maiamai038,
        Maiamai039,
        Maiamai040,
        Maiamai041,
        Maiamai042,
        Maiamai043,
        Maiamai044,
        Maiamai045,
        Maiamai046,
        Maiamai047,
        Maiamai048,
        Maiamai049,
        Maiamai050,
        Maiamai051,
        Maiamai052,
        Maiamai053,
        Maiamai054,
        Maiamai055,
        Maiamai056,
        Maiamai057,
        Maiamai058,
        Maiamai059,
        Maiamai060,
        Maiamai061,
        Maiamai062,
        Maiamai063,
        Maiamai064,
        Maiamai065,
        Maiamai066,
        Maiamai067,
        Maiamai068,
        Maiamai069,
        Maiamai070,
        Maiamai071,
        Maiamai072,
        Maiamai073,
        Maiamai074,
        Maiamai075,
        Maiamai076,
        Maiamai077,
        Maiamai078,
        Maiamai079,
        Maiamai080,
        Maiamai081,
        Maiamai082,
        Maiamai083,
        Maiamai084,
        Maiamai085,
        Maiamai086,
        Maiamai087,
        Maiamai088,
        Maiamai089,
        Maiamai090,
        Maiamai091,
        Maiamai092,
        Maiamai093,
        Maiamai094,
        Maiamai095,
        Maiamai096,
        Maiamai097,
        Maiamai098,
        Maiamai099,
        Maiamai100,
    ]
}

fn get_item_pools(settings: &Settings, rng: &mut StdRng) -> (Vec<FillerItem>, Vec<FillerItem>) {
    let dungeon_prizes = vec![
        PendantOfCourage,
        PendantOfWisdom,
        PendantOfPower,
        SageGulley,
        SageOren,
        SageSeres,
        SageOsfala,
        SageImpa,
        SageIrene,
        SageRosso,
    ];

    let big_keys = vec![
        EasternKeyBig,
        GalesKeyBig,
        HeraKeyBig,
        DarkKeyBig,
        SwampKeyBig,
        SkullKeyBig,
        ThievesKeyBig,
        IceKeyBig,
        DesertKeyBig,
        TurtleKeyBig,
    ];

    let small_keys = vec![
        HyruleSanctuaryKey,
        LoruleSanctuaryKey,
        EasternKeySmall01,
        EasternKeySmall02,
        GalesKeySmall01,
        GalesKeySmall02,
        GalesKeySmall03,
        GalesKeySmall04,
        HeraKeySmall01,
        HeraKeySmall02,
        DarkKeySmall01,
        DarkKeySmall02,
        DarkKeySmall03,
        DarkKeySmall04,
        SwampKeySmall01,
        SwampKeySmall02,
        SwampKeySmall03,
        SwampKeySmall04,
        SkullKeySmall01,
        SkullKeySmall02,
        SkullKeySmall03,
        ThievesKeySmall,
        IceKeySmall01,
        IceKeySmall02,
        IceKeySmall03,
        DesertKeySmall01,
        DesertKeySmall02,
        DesertKeySmall03,
        DesertKeySmall04,
        DesertKeySmall05,
        TurtleKeySmall01,
        TurtleKeySmall02,
        TurtleKeySmall03,
        LoruleCastleKeySmall01,
        LoruleCastleKeySmall02,
        LoruleCastleKeySmall03,
        LoruleCastleKeySmall04,
        LoruleCastleKeySmall05,
    ];

    let compasses = vec![
        EasternCompass,
        GalesCompass,
        HeraCompass,
        DarkCompass,
        SwampCompass,
        SkullCompass,
        ThievesCompass,
        IceCompass,
        DesertCompass,
        TurtleCompass,
        LoruleCastleCompass,
    ];

    let mut progression_items = vec![
        Bow01,
        Boomerang01,
        Hookshot01,
        Bombs01,
        FireRod01,
        IceRod01,
        Hammer01,
        SandRod01,
        TornadoRod01,
        RaviosBracelet01,
        RaviosBracelet02,
        Bell,
        StaminaScroll,
        BowOfLight,
        PegasusBoots,
        Flippers,
        HylianShield,
        PremiumMilk,
        SmoothGem,
        LetterInABottle,
        Lamp01,
        Net01,
        Pouch,

        // 5 Bottles
        Bottle01,
        Bottle02,
        Bottle03,
        Bottle04,
        Bottle05,

        // 2 Gloves
        Glove01,
        Glove02,

        // 2 Mails
        Mail01,
        Mail02,

        // 4 Master Ore
        OreYellow,
        OreGreen,
        OreBlue,
        OreRed,

        // 18 Purple Rupees
        RupeePurple01,
        RupeePurple02,
        RupeePurple03,
        RupeePurple04,
        RupeePurple05,
        RupeePurple06,
        RupeePurple07,
        RupeePurple08,
        RupeePurple09,
        RupeePurple10,
        RupeePurple11,
        RupeePurple12,
        RupeePurple13,
        RupeePurple14,
        RupeePurple15,
        RupeePurple16,
        RupeePurple17,
        RupeePurple18,

        // 38 Silver Rupees
        RupeeSilver01,
        RupeeSilver02,
        RupeeSilver03,
        RupeeSilver04,
        RupeeSilver05,
        RupeeSilver06,
        RupeeSilver07,
        RupeeSilver08,
        RupeeSilver09,
        RupeeSilver10,
        RupeeSilver11,
        RupeeSilver12,
        RupeeSilver13,
        RupeeSilver14,
        RupeeSilver15,
        RupeeSilver16,
        RupeeSilver17,
        RupeeSilver18,
        RupeeSilver19,
        RupeeSilver20,
        RupeeSilver21,
        RupeeSilver22,
        RupeeSilver23,
        RupeeSilver24,
        RupeeSilver25,
        RupeeSilver26,
        RupeeSilver27,
        RupeeSilver28,
        RupeeSilver29,
        RupeeSilver30,
        RupeeSilver31,
        RupeeSilver32,
        RupeeSilver33,
        RupeeSilver34,
        RupeeSilver35,
        RupeeSilver36,
        RupeeSilver37,
        RupeeSilver38,
        RupeeSilver39, // Cucco Dungeon
        RupeeSilver40, // Turtle Rock 1F
        //RupeeSilver41, // FIXME Ku's Domain Silver

        // 8 Gold Rupees
        RupeeGold01,
        RupeeGold02,
        RupeeGold03,
        RupeeGold04,
        RupeeGold05,
        RupeeGold06,
        RupeeGold07,
        RupeeGold08,
        RupeeGold09, // Dark Palace 2F
        RupeeGold10, // Turtle Rock B1
    ];

    progression_items.extend(maiamai_pool());

    let mut trash_pool = vec![
        HintGlasses,

        // 2 Green Rupees
        RupeeGreen,
        RupeeGreen,

        // 8 Blue Rupees
        RupeeBlue,
        RupeeBlue,
        RupeeBlue,
        RupeeBlue,
        RupeeBlue,
        RupeeBlue,
        RupeeBlue,
        RupeeBlue,

        // 19 Red Rupees
        RupeeRed,
        RupeeRed,
        RupeeRed,
        RupeeRed,
        RupeeRed,
        RupeeRed,
        RupeeRed,
        RupeeRed,
        RupeeRed,
        RupeeRed,
        RupeeRed,
        RupeeRed,
        RupeeRed,
        RupeeRed,
        RupeeRed,
        RupeeRed,
        RupeeRed,
        RupeeRed,
        RupeeRed,

        // 4 Monster Tails
        MonsterTail,
        MonsterTail,
        MonsterTail,
        MonsterTail,

        // 3 Monster Horns
        MonsterHorn,
        MonsterHorn,
        MonsterHorn,

        // 12 Monster Guts
        MonsterGuts,
        MonsterGuts,
        MonsterGuts,
        MonsterGuts,
        MonsterGuts,
        MonsterGuts,
        MonsterGuts,
        MonsterGuts,
        MonsterGuts,
        MonsterGuts,
        MonsterGuts,
        //MonsterGuts, // Removed for Bracelet #2

        // Heart Pieces
        HeartPiece01,
        HeartPiece02,
        HeartPiece03,
        HeartPiece04,
        HeartPiece05,
        HeartPiece06,
        HeartPiece07,
        HeartPiece08,
        HeartPiece09,
        HeartPiece10,
        HeartPiece11,
        HeartPiece12,
        HeartPiece13,
        HeartPiece14,
        HeartPiece15,
        HeartPiece16,
        HeartPiece17,
        HeartPiece18,
        HeartPiece19,
        HeartPiece20,
        HeartPiece21,
        HeartPiece22,
        HeartPiece23,
        HeartPiece24,
        HeartPiece25,
        HeartPiece26,
        HeartPiece27,
        // HeartPiece28, // Not yet randomized, in Fortune's Choice minigame

        // Heart Containers
        HeartContainer01,
        HeartContainer02,
        HeartContainer03,
        HeartContainer04,
        HeartContainer05,
        HeartContainer06,
        HeartContainer07,
        HeartContainer08,
        HeartContainer09,
        HeartContainer10,
    ];

    // Remove the Bee Badge from Hell Logic to keep Bee Boosting viable
    trash_pool.push(match settings.logic.mode {
        GlitchHell => MonsterHorn,
        _ => BeeBadge
    });

    // Swordless Mode
    if settings.logic.swordless_mode {
        trash_pool.push(Empty);
        trash_pool.push(Empty);
        trash_pool.push(Empty);
        trash_pool.push(Empty);
    } else {
        progression_items.push(Sword01);
        progression_items.push(Sword02);
        progression_items.push(Sword03);
        progression_items.push(Sword04);
    }

    // Super Items
    if settings.logic.super_items {
        progression_items.push(Lamp02);
        progression_items.push(Net02);
    } else {
        trash_pool.push(MonsterTail);
        trash_pool.push(MonsterTail);
    }

    (
        order_progression_pools(settings, rng, dungeon_prizes, big_keys, small_keys, compasses, progression_items),
        shuffle_items(trash_pool, rng)
    )
}


/**
 * Order the progression items for placement.
 *
 * For Keysanity seeds, just combine everything and shuffle.
 *
 * For non-Keysanity seeds this means shuffling categories of items amongst themselves, then ordering them:
 * - Dungeon Prizes
 * - Big Keys
 * - Small Keys
 * - Compasses
 * - All other progression
 */
fn order_progression_pools(_settings: &Settings,
                           rng: &mut StdRng,
                           dungeon_prizes: Vec<FillerItem>,
                           big_keys: Vec<FillerItem>,
                           small_keys: Vec<FillerItem>,
                           compasses: Vec<FillerItem>,
                           progression: Vec<FillerItem>) -> Vec<FillerItem> {
    let mut progression_pool;

    let is_keysanity = false; // No Keysanity yet, hardcode this to false
    if is_keysanity {
        // Combine all category pools and shuffle
        progression_pool = dungeon_prizes;
        progression_pool.extend(big_keys);
        progression_pool.extend(small_keys);
        progression_pool.extend(compasses);
        progression_pool.extend(progression);
        progression_pool = shuffle_items(progression_pool, rng);
    } else {
        // Shuffle respective category pools and order by category
        progression_pool = shuffle_items(dungeon_prizes, rng);
        progression_pool.extend(shuffle_items(big_keys, rng));
        progression_pool.extend(shuffle_items(small_keys, rng));
        progression_pool.extend(shuffle_items(compasses, rng));
        progression_pool.extend(shuffle_items(progression, rng));
    }

    progression_pool
}

/// Shuffles item pool to eliminate placement order bias
fn shuffle_items(mut items: Vec<FillerItem>, rng: &mut StdRng) -> Vec<FillerItem> {
    let mut shuffled_items: Vec<FillerItem> = Vec::new();

    while !items.is_empty() {
        shuffled_items.push(items.remove(rng.gen_range(0..items.len())));
    }

    shuffled_items
}

fn is_dungeon_prize(item: FillerItem) -> bool {
    match item {
        PendantOfPower |
        PendantOfWisdom |
        PendantOfCourage |
        SageGulley |
        SageOren |
        SageSeres |
        SageOsfala |
        SageImpa |
        SageIrene |
        SageRosso => true,
        _ => false
    }
}

fn is_dungeon_item(item: FillerItem) -> bool {
    match item {
        HyruleSanctuaryKey |
        LoruleSanctuaryKey |
        EasternCompass |
        EasternKeyBig |
        EasternKeySmall01 |
        EasternKeySmall02 |
        GalesCompass |
        GalesKeyBig |
        GalesKeySmall01 |
        GalesKeySmall02 |
        GalesKeySmall03 |
        GalesKeySmall04 |
        HeraCompass |
        HeraKeyBig |
        HeraKeySmall01 |
        HeraKeySmall02 |
        DarkCompass |
        DarkKeyBig |
        DarkKeySmall01 |
        DarkKeySmall02 |
        DarkKeySmall03 |
        DarkKeySmall04 |
        SwampCompass |
        SwampKeyBig |
        SwampKeySmall01 |
        SwampKeySmall02 |
        SwampKeySmall03 |
        SwampKeySmall04 |
        SkullCompass |
        SkullKeyBig |
        SkullKeySmall01 |
        SkullKeySmall02 |
        SkullKeySmall03 |
        ThievesCompass |
        ThievesKeyBig |
        ThievesKeySmall |
        IceCompass |
        IceKeyBig |
        IceKeySmall01 |
        IceKeySmall02 |
        IceKeySmall03 |
        DesertCompass |
        DesertKeyBig |
        DesertKeySmall01 |
        DesertKeySmall02 |
        DesertKeySmall03 |
        DesertKeySmall04 |
        DesertKeySmall05 |
        TurtleCompass |
        TurtleKeyBig |
        TurtleKeySmall01 |
        TurtleKeySmall02 |
        TurtleKeySmall03 |
        LoruleCastleCompass |
        LoruleCastleKeySmall01 |
        LoruleCastleKeySmall02 |
        LoruleCastleKeySmall03 |
        LoruleCastleKeySmall04 |
        LoruleCastleKeySmall05 => true,
        _ => false,
    }
}

fn fill_trash(check_map: &mut HashMap<&str, Option<FillerItem>>, rng: &mut StdRng, trash_items: &Vec<FillerItem>) {
    info!("Placing Junk Items...");

    let mut empty_check_keys = Vec::new();
    for (key, val) in check_map.clone() {
        if val.is_none() {
            empty_check_keys.push(key);
        }
    }

    if empty_check_keys.len() != trash_items.len() {
        error!("Number of empty checks: {} does not match available trash items: {}", empty_check_keys.len(), trash_items.len());
        exit(1);
    }

    for trash in trash_items {
        check_map.insert(empty_check_keys.remove(rng.gen_range(0..empty_check_keys.len())), Some(*trash));
    }
}

fn place_item_randomly(item: FillerItem, checks: &Vec<Check>, check_map: &mut HashMap<&str, Option<FillerItem>>, rng: &mut StdRng) {
    let index = rng.gen_range(0..checks.len());
    check_map.insert(checks.get(index).unwrap().get_name(), Some(item));
}

fn filter_checks(item: FillerItem, checks: &mut Vec<Check>, check_map: &mut HashMap<&str, Option<FillerItem>>) -> Vec<Check> {

    // Filter out non-empty checks
    let mut filtered_checks = checks.iter().filter(|&x| check_map.get(x.get_name()).unwrap().is_none()).cloned().collect();

    // Filter checks by item type
    if is_dungeon_prize(item) {
        filtered_checks = filter_dungeon_prize_checks(&mut filtered_checks);
    } else if is_dungeon_item(item) {
        let is_keysanity = false; // No keysanity yet, hardcode to false
        if !is_keysanity {
            filtered_checks = filter_dungeon_checks(item, &mut filtered_checks);
        }
    }

    filtered_checks
}

fn filter_dungeon_prize_checks(eligible_checks: &mut Vec<Check>) -> Vec<Check> {
    eligible_checks.iter().filter(|&x| x.get_name().contains("Prize")).cloned().collect()
}

fn filter_dungeon_checks(item: FillerItem, eligible_checks: &mut Vec<Check>) -> Vec<Check> {
    eligible_checks.iter().filter(|&x| x.get_name().starts_with(match item {
        HyruleSanctuaryKey => "[HS]",
        LoruleSanctuaryKey => "[LS]",

        EasternCompass | EasternKeyBig | EasternKeySmall01 | EasternKeySmall02 => "[EP]",
        GalesCompass | GalesKeyBig | GalesKeySmall01 | GalesKeySmall02 | GalesKeySmall03 | GalesKeySmall04 => "[HG]",
        HeraCompass | HeraKeyBig | HeraKeySmall01 | HeraKeySmall02 => "[TH]",

        DarkCompass | DarkKeyBig | DarkKeySmall01 | DarkKeySmall02 | DarkKeySmall03 | DarkKeySmall04 => "[PD]",
        SwampCompass | SwampKeyBig | SwampKeySmall01 | SwampKeySmall02 | SwampKeySmall03 | SwampKeySmall04 => "[SP]",
        SkullCompass | SkullKeyBig | SkullKeySmall01 | SkullKeySmall02 | SkullKeySmall03 => "[SW]",
        ThievesCompass | ThievesKeyBig | ThievesKeySmall => "[T'H]",
        IceCompass | IceKeyBig | IceKeySmall01 | IceKeySmall02 | IceKeySmall03 => "[IR]",
        DesertCompass | DesertKeyBig | DesertKeySmall01 | DesertKeySmall02 | DesertKeySmall03 | DesertKeySmall04 | DesertKeySmall05 => "[DP]",
        TurtleCompass | TurtleKeyBig | TurtleKeySmall01 | TurtleKeySmall02 | TurtleKeySmall03 => "[TR]",

        LoruleCastleCompass | LoruleCastleKeySmall01 | LoruleCastleKeySmall02 | LoruleCastleKeySmall03 | LoruleCastleKeySmall04 | LoruleCastleKeySmall05 => "[LC]",

        _ => { panic!("Item {:?} is not a dungeon item", item); }
    })).cloned().collect()
}

fn exist_empty_reachable_check(checks: &Vec<Check>, check_map: &HashMap<&str, Option<FillerItem>>) -> bool {
    for check in checks {
        match check_map.get(check.get_name()).unwrap() {
            None => { return true; }
            Some(_) => {}
        }
    }

    false
}

/// Prefills a map with all checks as defined by the world graph with no values yet assigned
fn prefill_check_map(world_graph: &mut HashMap<Location, LocationNode>) -> HashMap<&'static str, Option<FillerItem>> {
    let mut check_map = HashMap::new();

    for (_, location_node) in world_graph {
        for check in location_node.clone().get_checks() {
            if check_map.insert(check.get_name(), match check.get_quest() {
                None => None,
                Some(quest) => Some(quest) // Quest items are static so just set them right away
            }).is_some() {
                error!("Multiple checks have duplicate name: {}", check.get_name());
                exit(1);
            }
        }
    }

    check_map
}

/// This translation is probably adding unnecessary overhead, oh well
fn build_progress_from_items(items: &Vec<FillerItem>, settings: &Settings) -> Progress {
    let mut progress = Progress::new(settings.clone());
    for item in items {
        progress.add_item(*item);
    }

    progress
}

fn verify_all_locations_accessible(loc_map: &mut HashMap<Location, LocationNode>,
                                   progression_pool: &Vec<FillerItem>,
                                   settings: &Settings) {
    info!("Verifying all locations accessible...");

    let mut check_map = prefill_check_map(loc_map);

    let reachable_checks = assumed_search(loc_map, progression_pool, &mut check_map, settings); //find_reachable_checks(loc_map, &everything, &mut check_map); //

    const TOTAL_CHECKS: usize =
        255 // Standard
            + 10 // Dungeon Prizes
            + 100 // Maiamai
            + 28; // Quest;

    if reachable_checks.len() != TOTAL_CHECKS {

        // for rc in &reachable_checks {
        //     info!("Reachable Check: {}", rc.get_name());
        // }

        error!("Only {}/{} checks were reachable in the world graph", reachable_checks.len(), TOTAL_CHECKS);
        exit(1);
    }
}

/// Find all checks reachable with the given Progress
fn find_reachable_checks(loc_map: &mut HashMap<Location, LocationNode>, progress: &Progress) -> Vec<Check> {
    let start_node = Location::RavioShop;
    let mut loc_queue: Queue<Location> = Queue::from(vec![start_node]);
    let mut visited: HashSet<Location> = HashSet::new();
    let mut reachable_checks: Vec<Check> = Vec::new(); // possibly switch to HashSet to avoid dupes

    visited.insert(start_node);

    while !loc_queue.is_empty() {
        let location = loc_queue.dequeue().unwrap();

        // Grab the location from the map, verify it is defined
        let location_node = match loc_map.get_mut(&location) {
            Some(loc) => loc,
            None => {
                info!("Location Undefined: {:?}", location);
                exit(1);
            }
        };

        // Iterate over the location's checks
        for check in location_node.clone().get_checks() {
            if check.can_access(progress) {
                reachable_checks.push(check);
            }
        }

        // Queue new paths reachable from this location
        for path in location_node.clone().get_paths() {
            let destination = path.get_destination();
            if !visited.contains(&destination) && path.can_access(progress) {
                loc_queue.queue(destination).expect("TODO: panic message");
                visited.insert(destination);
            }
        }
    }

    reachable_checks
}

fn get_items_from_reachable_checks(reachable_checks: &Vec<Check>,
                                   check_map: &mut HashMap<&str, Option<FillerItem>>,
                                   settings: &Settings) -> Progress {
    let mut progress = Progress::new(settings.clone());

    for check in reachable_checks {

        // Items already placed in the world that can be picked up
        let placed_item = check_map.get(check.get_name()).unwrap();
        match placed_item {
            None => {}
            Some(item) => progress.add_item(*item)
        }

        // Quest items that will always be at a given check
        match check.get_quest() {
            None => {}
            Some(quest) => { progress.add_item(quest) }
        }
    }

    progress
}

/// The Assumed Fill algorithm
///
/// Randomly places `items_owned` into the `check_map` in a completable manner as informed by the
/// logic defined in the `world_graph` and `settings`.
///
/// Items are placed "backwards", *assuming* that all items that have yet to be placed are
/// available without the item currently being placed.
///
/// An assumed search algorithm is used to identify all locations reachable without the item
/// currently being placed.
///
/// * `world_graph` - A graph representing the comprehensive structure of the game world
/// * `rng` - The RNG seed
/// * `items_owned` - The pool of all progression-granting items
/// * `check_map` - A map representing all checks and items assigned to them
/// * `settings` - Game settings
fn assumed_fill(mut world_graph: &mut HashMap<Location, LocationNode>,
                mut rng: &mut StdRng,
                items_owned: &mut Vec<FillerItem>,
                mut check_map: &mut HashMap<&str, Option<FillerItem>>,
                settings: &Settings) {
    info!("Placing Progression Items...");

    let mut reachable_checks = assumed_search(&mut world_graph, &items_owned, &mut check_map, settings);

    while exist_empty_reachable_check(&reachable_checks, &check_map) && !items_owned.is_empty() {
        let item = items_owned.remove(0);

        //
        reachable_checks = assumed_search(&mut world_graph, &items_owned, &mut check_map, settings);

        let filtered_checks = filter_checks(item, &mut reachable_checks, &mut check_map);

        if filtered_checks.len() == 0 {
            info!("No reachable checks found to place: {:?}", item);
        }

        place_item_randomly(item, &filtered_checks, &mut check_map, &mut rng);
    }
}

/// The Assumed Search algorithm.
///
/// Gets all reachable checks available with the `items_owned`, assuming all items yet to be
/// placed will be available.
///
/// A loop is performed to expand the considered items to include not just the `items_owned` but
/// also all items already placed that are reachable with the currently considered items, until
/// all such items have been exhausted.
///
fn assumed_search(loc_map: &mut HashMap<Location, LocationNode>,
                  items_owned: &Vec<FillerItem>,
                  mut check_map: &mut HashMap<&str, Option<FillerItem>>,
                  settings: &Settings) -> Vec<Check> {
    let mut considered_items = build_progress_from_items(&items_owned.clone(), settings);
    let mut reachable_checks: Vec<Check>;

    loop {
        reachable_checks = find_reachable_checks(loc_map, &considered_items);
        let reachable_items = get_items_from_reachable_checks(&reachable_checks, &mut check_map, settings);

        let new_items = reachable_items.difference(&considered_items);

        if new_items.is_empty() {
            break;
        }

        for new_item in new_items {
            considered_items.add_item(new_item);
        }
    }

    reachable_checks
}









