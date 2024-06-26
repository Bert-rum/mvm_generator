use clap::Parser;
use evalexpr::{HashMapContext, eval_empty_with_context_mut, eval_int_with_context_mut, eval_with_context_mut,eval_float_with_context_mut, Value::Int, Value::Float};
use std::fs::{self, File};
use std::io::Write;
use std::time::Instant;
use rand::seq::SliceRandom;
use rayon::prelude::*;
use rand::prelude::*;
mod bot;
mod mission;
mod wavespawn;
use crate::bot::Bot;
use crate::mission::Mission;
use crate::wavespawn::Wavespawn;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    map: String,
    #[arg(short, long)]
    name: String,
    #[arg(short, long)]
    config: String,
}
//Hierarchy: Mission -> Waves -> Wavespawns -> Bots
//Test: cargo run -- -m mvm_decoy -n lol -s 10000 -c normal_if.json
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let beginning = Instant::now();
    
    let args = Args::parse();
    let mut mission: Mission = Mission {
        ..Default::default()
    };
    let mut bots: Vec<Bot> = vec![];
    let mut wavespawns: Vec<Wavespawn> = vec![];

    mission.parse_mission_config(&args.config);
    mission.parse_map_config(&args.map);

    //free up memory on exit scope
    {
        let now = Instant::now();

        let bot_config = fs::read_to_string("./config/bots.json")?;
        let bot_info_string: serde_json::Value = serde_json::from_str(&bot_config)?;
        let bot_infos = &bot_info_string.as_object().unwrap();
        for value in *bot_infos {
            let mut new_bot: Bot = Bot {
                name: value.0.to_string(),
                class: match value.1["class"].as_str() {
                    None => "scout".to_string(),
                    Some(val) => val.to_string(),
                },
                class_icon: match value.1["class_icon"].as_str() {
                    None => "".to_string(),
                    Some(val) => val.to_string(),
                },
                weapons: match value.1["weapons"].as_array() {
                    None => vec![],
                    Some(val) => val.iter().map(|x| x.as_str().unwrap().to_owned()).collect(),
                },
                difficulty: match value.1["difficulty"].as_i64() {
                    None => 1,
                    Some(val) => val,
                },
                weapon_restriction: match value.1["weapon_restriction"].as_str() {
                    None => "".to_string(),
                    Some(val) => val.to_string(),
                },
                behavior: match value.1["behavior"].as_str() {
                    None => "".to_string(),
                    Some(val) => val.to_string(),
                },
                bot_attributes: match value.1["bot_attributes"].as_array() {
                    None => vec![],
                    Some(val) => val.iter().map(|x| x.as_str().unwrap().to_owned()).collect(),
                },
                tags: match value.1["tags"].as_array() {
                    None => vec![],
                    Some(val) => val.iter().map(|x| x.as_str().unwrap().to_owned()).collect(),
                },
                health: match value.1["health"].as_str() {
                    None => "".to_string(),
                    Some(val) => val.to_string(),
                },
                scale: match value.1["scale"].as_f64() {
                    None => 1.0,
                    Some(val) => val,
                },
                max_vision_range: match value.1["max_vision_range"].as_i64() {
                    None => 0,
                    Some(val) => val,
                },
                auto_jump_min: match value.1["auto_jump_min"].as_i64() {
                    None => 0,
                    Some(val) => val,
                },
                auto_jump_max: match value.1["auto_jump_max"].as_i64() {
                    None => 0,
                    Some(val) => val,
                },
                is_boss: match value.1["is_boss"].as_bool() {
                    None => false,
                    Some(val) => val,
                },
                is_giant: match value.1["is_giant"].as_bool() {
                    None => false,
                    Some(val) => val,
                },
                is_gatebot: match value.1["is_gatebot"].as_bool() {
                    None => false,
                    Some(val) => val,
                },
                is_mission_bot: match value.1["is_mission_bot"].as_bool() {
                    None => false,
                    Some(val) => val,
                },
                currency_weight: match value.1["currency_weight"].as_i64() {
                    None => 1,
                    Some(val) => val,
                },
                count: match value.1["count"].as_i64() {
                    None => 10,
                    Some(val) => val,
                },
                max_active: match value.1["max_active"].as_i64() {
                    None => 10,
                    Some(val) => val,
                },
                spawn_per_timer: match value.1["spawn_per_timer"].as_i64() {
                    None => 2,
                    Some(val) => val,
                },
                time_before_spawn: match value.1["time_before_spawn"].as_i64() {
                    None => 0,
                    Some(val) => val,
                },
                time_between_spawn: match value.1["time_between_spawn"].as_i64() {
                    None => 5,
                    Some(val) => val,
                },
                attributes: match value.1["attributes"].as_array() {
                    None => vec![],
                    Some(val) => val
                        .iter()
                        .map(|x| match x.as_array() {
                            Some(inner) => [
                                inner[0].as_str().unwrap().to_owned(),
                                inner[1].as_str().unwrap().to_owned(),
                            ],
                            None => panic!("WTF! - Failed to parse attributes for {}", value.0.to_string()),
                        })
                        .collect(),
                },
                weapon_attributes: match value.1["weapon_attributes"].as_array() {
                    None => vec![],
                    Some(val) => val
                        .iter()
                        .map(|x| match x.as_array() {
                            Some(inner) => [
                                inner[0].as_str().unwrap().to_owned(),
                                inner[1].as_str().unwrap().to_owned(),
                            ],
                            None => panic!("WTF! - Failed to parse attributes for {}", value.0.to_string()),
                        })
                        .collect(),
                }
            };
            if new_bot.class_icon.is_empty(){
                new_bot.class_icon = new_bot.class.to_string();
            }
            bots.push(new_bot);
        }
        println!("took {:?} to parse bot config", now.elapsed());
    }
    {
        let now = Instant::now();

        let bot_config = fs::read_to_string("./config/wavespawns.json")?;
        let squad_info_string: serde_json::Value = serde_json::from_str(&bot_config)?;
        let squad_infos = &squad_info_string.as_object().unwrap();
        for value in *squad_infos {
            let wavespawn: Wavespawn = Wavespawn {
                squads: match value.1["squads"].as_array() {
                    None => vec![],
                    Some(val) => val.iter().map(|x| match bots.iter().find(|y| *y.name == x.as_str().unwrap().to_owned()) {
                        Some(valid) => {valid},
                        None => {panic!("Invalid bot name: {}", x.as_str().unwrap())},
                    } ).cloned().collect(),
                },
                tags: match value.1["tags"].as_array() {
                    None => vec![],
                    Some(val) => val.iter().map(|x| x.as_str().unwrap().to_owned()).collect(),
                },
                weight: match value.1["weight"].as_f64() {
                    None => 1.0,
                    Some(val) => val,
                },
                rarity: match value.1["rarity"].as_f64() {
                    None => 1.0,
                    Some(val) => val,
                },
                spawn_tank: match value.1["with_tank"].as_bool() {
                    None => false,
                    Some(val) => val,
                },
            };
            for tag in &mission.wavespawn_tags{
                if wavespawn.tags.contains(tag){
                    wavespawns.push(wavespawn);
                    break;
                }
            }
        }
        println!("took {:?} to parse wavespawn config", now.elapsed());
    }
    let bot_wavespawns: Vec<Wavespawn> = wavespawns.clone().into_par_iter()
        .filter(|i| !( i.tags.contains(&String::from("giant")) || i.tags.contains(&String::from("boss")) || i.tags.contains(&String::from("superboss"))) ).collect();
    let giant_wavespawns: Vec<Wavespawn> = wavespawns.clone().into_par_iter().filter(|i| i.tags.contains(&String::from("giant"))).collect();
    let boss_wavespawns: Vec<Wavespawn> = wavespawns.clone().into_par_iter().filter(|i| i.tags.contains(&String::from("boss"))).collect();
    let superboss_wavespawns: Vec<Wavespawn> = wavespawns.clone().into_par_iter().filter(|i| i.tags.contains(&String::from("superboss"))).collect();
    let mission_bots: Vec<Bot> = bots.clone().into_par_iter().filter(|i| i.is_mission_bot).collect();
    let non_engineer_mission_bots: Vec<Bot> = bots.clone().into_par_iter().filter(|i| i.is_mission_bot && i.class != "engineer").collect();

    //Wave Generation Process
    let mut pop_file = String::new();
    let mut output_file = File::create("./output/".to_string()+&args.map+"_"+&mission.mission_name+".pop")?;

    //Write the header of the file
    //pop_file.push_str("#base robot_giant.pop\n#base robot_standard.pop\n#base robot_gatebot.pop\n");
    pop_file.push_str("WaveSchedule\n{\n");
    pop_file.push_str("\tRespawnWaveTime 8\n");
    pop_file.push_str("\tCanBotsAttackWhileInSpawnRoom No\n");
    pop_file.push_str("\tFixedRespawnWaveTime Yes\n");
    pop_file.push_str("\tAdvanced 1\n");
    pop_file.push_str(&format!("\tStartingCurrency\t{}\n", mission.starting_money));

    //Each wave gets its own thread.
    let generation: String = (1..mission.wave_amount+1).into_par_iter().map(|i| {
        let mut wave_portion: String = String::new();
        let mut context = HashMapContext::new();
        eval_empty_with_context_mut(&format!("wave = {}",i), &mut context).unwrap();

        let money_for_wave: f64 = eval_float_with_context_mut(&mission.money_per_wave, &mut context).unwrap();
        let wave_rarity = eval_float_with_context_mut(&mission.rarity_formula, &mut context).unwrap();

        //stupid wave boilerplate shit

        let mut bot_id: i64 = 0;
        let mut last_bot: i64 = 0;
        let mut rng = thread_rng();

        if rng.gen_bool(0.75){
            let chosen_bot: &Bot;
            if mission.engineers_enabled {
                chosen_bot = mission_bots.choose(&mut rng).unwrap();
            }else{
                chosen_bot = non_engineer_mission_bots.choose(&mut rng).unwrap();
            }
            wave_portion.push_str("\tMission\n\t{\n");
            wave_portion.push_str(&format!("\t\tObjective\t{}\n", chosen_bot.class));
            wave_portion.push_str(&format!("\t\tInitialCooldown\t{}\n", rng.gen_range(50..120)));
            wave_portion.push_str(&format!("\t\tCooldownTime\t{}\n", rng.gen_range(30..75)));
            wave_portion.push_str(&format!("\t\tDesiredCount\t{}\n", rng.gen_range(2..4)));
            for area in &mission.spawn_support_areas{
                wave_portion.push_str(&format!("\t\tWhere\t{}\n", area));
            }
            wave_portion.push_str(&format!("\t\tBeginAtWave\t{}\n", i));
            wave_portion.push_str("\t\tRunForThisManyWaves\t1\n");

            wave_portion.push_str("\t\tTFBot\n\t\t{\n");
            wave_portion.push_str(&format!("\t\t\tClassIcon\t\"{}\"\n",chosen_bot.class_icon));

            let eval_health = eval_int_with_context_mut(&chosen_bot.health, &mut context).unwrap();
            wave_portion.push_str(&format!("\t\t\tHealth\t{}\n",eval_health));
            wave_portion.push_str(&format!("\t\t\tName\t\"{}\"\n",chosen_bot.name));
            wave_portion.push_str(&format!("\t\t\tClass\t\"{}\"\n",chosen_bot.class));
            if chosen_bot.scale != 1.0 {
                wave_portion.push_str(&format!("\t\t\tScale\t\"{}\"\n",chosen_bot.scale));
            }
            let difficulty = match chosen_bot.difficulty {
                2 => "Normal",
                3 => "Hard",
                4 => "Expert",
                _ => "Easy"
            };
            wave_portion.push_str(&format!("\t\t\tSkill\t{}\n",difficulty));
            if !chosen_bot.weapon_restriction.is_empty(){
                wave_portion.push_str(&format!("\t\t\tWeaponRestrictions\t{}\n", chosen_bot.weapon_restriction));
            }
            for bot_attribute in &chosen_bot.bot_attributes{
                wave_portion.push_str(&format!("\t\t\tAttributes\t\"{}\"\n", bot_attribute));
            }
            if chosen_bot.is_boss {
                wave_portion.push_str("\t\t\tAttributes\t\"UseBossHealthBar\"\n");
            }
            if chosen_bot.auto_jump_min != 0{
                wave_portion.push_str(&format!("\t\t\tAutoJumpMin\t{}\n", chosen_bot.auto_jump_min));
            }
            if chosen_bot.auto_jump_max != 0{
                wave_portion.push_str(&format!("\t\t\tAutoJumpMax\t{}\n", chosen_bot.auto_jump_max));
            }
            if chosen_bot.max_vision_range != 0{
                wave_portion.push_str(&format!("\t\t\tMaxVisionRange\t{}\n", chosen_bot.max_vision_range));
            }
            if chosen_bot.is_giant || chosen_bot.is_boss {
                wave_portion.push_str("\t\t\tAttributes\t\"MiniBoss\"\n");
                wave_portion.push_str(&format!("\t\t\tTag\t\"bot_giant\"\n"));
            }
            parse_bot_attributes(chosen_bot, &mission, &mut wave_portion, &mut context);
            if !chosen_bot.behavior.is_empty(){
                wave_portion.push_str(&format!("\t\t\tBehaviorModifiers\t{}\n", chosen_bot.behavior));
            }

            wave_portion.push_str("\t\t}\n");
            wave_portion.push_str("\t}\n");
        }

        wave_portion.push_str("\tWave\n\t{\n");

        wave_portion.push_str("\t\tCheckpoint\t\tYes\n");
        wave_portion.push_str("\t\tWaitWhenDone\t\t60\n");

        if mission.classic_relay  {
            wave_portion.push_str("\t\tStartWaveOutput\n\t\t{\n\t\t\tTarget\twave_start_relay_classic\n\t\t\tAction\tTrigger\n\t\t}\n");
        }
        else{
            wave_portion.push_str(&format!("\t\tStartWaveOutput\n\t\t{{\n\t\t\tTarget\t{}\n\t\t\tAction\tTrigger\n\t\t}}\n", mission.relayname));
        }

        if mission.gatebots_enabled {
            wave_portion.push_str("\t\tInitWaveOutput\n\t\t{\n\t\t\tTarget\tholograms_3way_relay\n\t\t\tAction\tTrigger\n\t\t}\n");
        }
        else{
            wave_portion.push_str("\t\tDoneOutput\n\t\t{\n\t\t\tTarget\twave_finished_relay\n\t\t\tAction\tTrigger\n\t\t}\n");
        }
        
        //Wavespawn + Currency Weight
        let mut finalized_spawns: Vec<&Wavespawn> = vec![];
        let mut total_weight: i64 = 0;
        if i >= mission.bot_superboss_waves {
            let chosen_wavespawn: &Wavespawn = superboss_wavespawns.choose_weighted(&mut rand::thread_rng(), |item| (item.weight*(1.0-((item.rarity-wave_rarity).abs())/wave_rarity)).clamp(0.0, 1000.0) ).unwrap();
            for chosen_bot in &chosen_wavespawn.squads{
                total_weight += chosen_bot.currency_weight;
            }
            finalized_spawns.push(chosen_wavespawn);
        }
        else if i >= mission.bot_boss_waves {
            let chosen_wavespawn: &Wavespawn = boss_wavespawns.choose_weighted(&mut rand::thread_rng(), |item| (item.weight*(1.0-((item.rarity-wave_rarity).abs())/wave_rarity)).clamp(0.0, 1000.0) ).unwrap();
            for chosen_bot in &chosen_wavespawn.squads{
                total_weight += chosen_bot.currency_weight;
            }
            finalized_spawns.push(chosen_wavespawn);
        }
        for _squad_num in 1..mission.wavespawn_amount+1{
            if rand::thread_rng().gen::<f64>() > mission.bot_giant_chance {
                let chosen_wavespawn = bot_wavespawns.choose_weighted(&mut rand::thread_rng(), |item| (item.weight*(1.0-((item.rarity-wave_rarity).abs())/wave_rarity)).clamp(0.0, 1000.0) ).unwrap();
                for chosen_bot in &chosen_wavespawn.squads{
                    total_weight += chosen_bot.currency_weight;
                }
                if chosen_wavespawn.spawn_tank{
                    total_weight += 20;
                }
                finalized_spawns.push(chosen_wavespawn);
            }else{
                let chosen_wavespawn = giant_wavespawns.choose_weighted(&mut rand::thread_rng(), |item| (item.weight*(1.0-((item.rarity-wave_rarity).abs())/wave_rarity)).clamp(0.0, 1000.0) ).unwrap();
                for chosen_bot in &chosen_wavespawn.squads{
                    total_weight += chosen_bot.currency_weight;
                }
                if chosen_wavespawn.spawn_tank{
                    total_weight += 20;
                }
                finalized_spawns.push(chosen_wavespawn);
            }
        }
        for wavespawn in finalized_spawns{
            for bot in &wavespawn.squads{
                wave_portion.push_str("\t\tWaveSpawn\n\t\t{\n");

                if wavespawn.tags.contains(&"support".to_string()){
                    wave_portion.push_str("\t\t\tSupport\t1\n");
                    wave_portion.push_str(&format!("\t\t\tName\t\"w{}_bs{}\"\n", i, bot_id));
                }else{
                    bot_id += 1;
                    wave_portion.push_str(&format!("\t\t\tName\t\"w{}_b{}\"\n", i, bot_id));
                }
                if last_bot != 0 && !wavespawn.tags.contains(&"support".to_string()){
                    wave_portion.push_str(&format!("\t\t\tWaitForAllDead\t\"w{}_b{}\"\n", i, last_bot));
                }

                wave_portion.push_str(&format!("\t\t\tTotalCount\t{}\n", bot.count));
                wave_portion.push_str(&format!("\t\t\tMaxActive\t{}\n", bot.max_active));
                wave_portion.push_str(&format!("\t\t\tSpawnCount\t{}\n", bot.spawn_per_timer));
                wave_portion.push_str(&format!("\t\t\tWaitBeforeStarting\t{}\n", bot.time_before_spawn));
                wave_portion.push_str(&format!("\t\t\tWaitBetweenSpawns\t{}\n", bot.time_between_spawn));



                if mission.gatebots_enabled {
                    if bot.is_boss{
                        for area in &mission.spawn_boss_areas{
                            wave_portion.push_str(&format!("\t\t\tWhere\t{}\n", area));
                        }
                    }
                    else if bot.is_giant{
                        for area in &mission.spawn_giants_areas{
                            wave_portion.push_str(&format!("\t\t\tWhere\t{}\n", area));
                        }
                    }
                    else {
                        for area in &mission.spawn_bot_areas{
                            wave_portion.push_str(&format!("\t\t\tWhere\t{}\n", area));
                        }
                    }
                }
                else{
                    if bot.is_boss{
                        wave_portion.push_str(&format!("\t\t\tWhere\t{}\n", mission.spawn_boss_areas.choose(&mut rng).unwrap()));
                    }
                    else if bot.is_giant{
                        wave_portion.push_str(&format!("\t\t\tWhere\t{}\n", mission.spawn_giants_areas.choose(&mut rng).unwrap()));
                    }
                    else {
                        wave_portion.push_str(&format!("\t\t\tWhere\t{}\n", mission.spawn_bot_areas.choose(&mut rng).unwrap()));
                    }
                }
                wave_portion.push_str(&format!("\t\t\tTotalCurrency\t{:.0}\n", bot.currency_weight as f64 / total_weight as f64 * money_for_wave as f64 ));

                wave_portion.push_str("\t\t\tSquad\n\t\t\t{\n\t\t\t\tTFBot\n\t\t\t\t{\n");
                wave_portion.push_str(&format!("\t\t\t\t\tClassIcon\t\"{}\"\n",bot.class_icon));

                let eval_health = eval_int_with_context_mut(&bot.health, &mut context).unwrap();
                wave_portion.push_str(&format!("\t\t\t\t\tHealth\t{}\n",eval_health));
                wave_portion.push_str(&format!("\t\t\t\t\tName\t\"{}\"\n",bot.name));
                wave_portion.push_str(&format!("\t\t\t\t\tClass\t\"{}\"\n",bot.class));
                if bot.scale != 1.0 {
                    wave_portion.push_str(&format!("\t\t\t\t\tScale\t\"{}\"\n",bot.scale));
                }
                let difficulty = match bot.difficulty {
                    2 => "Normal",
                    3 => "Hard",
                    4 => "Expert",
                    _ => "Easy"
                };
                wave_portion.push_str(&format!("\t\t\t\t\tSkill\t{}\n",difficulty));
                if !bot.weapon_restriction.is_empty(){
                    wave_portion.push_str(&format!("\t\t\t\t\tWeaponRestrictions\t{}\n", bot.weapon_restriction));
                }
                for bot_attribute in &bot.bot_attributes{
                    wave_portion.push_str(&format!("\t\t\t\t\tAttributes\t\"{}\"\n", bot_attribute));
                }
                if bot.is_boss {
                    wave_portion.push_str("\t\t\t\t\tAttributes\t\"UseBossHealthBar\"\n");
                }
                if bot.auto_jump_min != 0{
                    wave_portion.push_str(&format!("\t\t\t\t\tAutoJumpMin\t{}\n", bot.auto_jump_min));
                }
                if bot.auto_jump_max != 0{
                    wave_portion.push_str(&format!("\t\t\t\t\tAutoJumpMax\t{}\n", bot.auto_jump_max));
                }
                if bot.max_vision_range != 0{
                    wave_portion.push_str(&format!("\t\t\t\t\tMaxVisionRange\t{}\n", bot.max_vision_range));
                }
                if bot.is_giant || bot.is_boss {
                    wave_portion.push_str("\t\t\t\t\tAttributes\t\"MiniBoss\"\n");
                    wave_portion.push_str(&format!("\t\t\t\t\tTag\t\"bot_giant\"\n"));

                    parse_bot_attributes(bot, &mission, &mut wave_portion, &mut context);
                }
                else if mission.gatebots_enabled && (rng.gen_bool(0.5) || bot.is_gatebot) { //50% of bots or they are gatebot supports
                    wave_portion.push_str("\t\t\t\t\tEventChangeAttributes\n\t\t\t\t\t{\n");
                    wave_portion.push_str("\t\t\t\t\t\tDefault\n\t\t\t\t\t\t{\n");
                    wave_portion.push_str(&format!("\t\t\t\t\t\t\tItem\t\"{}\"\n", give_gatebot_cap_name(&bot.class)));
                    wave_portion.push_str("\t\t\t\t\t\t\tTag\tnav_prefer_gate1_flank\n");
                    wave_portion.push_str("\t\t\t\t\t\t\tTag\tbot_gatebot\n");
                    wave_portion.push_str("\t\t\t\t\t\t\tBehaviorModifiers\tpush\n");
                    wave_portion.push_str("\t\t\t\t\t\t\tAttributes\tIgnoreFlag\n");
                    parse_bot_attributes(bot, &mission, &mut wave_portion, &mut context);
                    wave_portion.push_str("\t\t\t\t\t\t}\n");
                    wave_portion.push_str("\t\t\t\t\t\tRevertGateBotsBehavior\n\t\t\t\t\t\t{\n");
                    wave_portion.push_str(&format!("\t\t\t\t\t\t\tItem\t\"{}\"\n", give_gatebot_cap_name(&bot.class)));
                    wave_portion.push_str("\t\t\t\t\t\t\tItemAttributes\n\t\t\t\t\t\t\t{\n");
                    wave_portion.push_str(&format!("\t\t\t\t\t\t\t\tItemName\t\"{}\"\n", give_gatebot_cap_name(&bot.class)));
                    wave_portion.push_str("\t\t\t\t\t\t\t\t\"item style override\"\t1\n");
                    wave_portion.push_str("\t\t\t\t\t\t\t}\n");
                    
                    if !bot.behavior.is_empty(){
                        wave_portion.push_str(&format!("\t\t\t\t\tBehaviorModifiers\t{}\n", bot.behavior));
                    }
                    parse_bot_attributes(bot, &mission, &mut wave_portion, &mut context);
                    wave_portion.push_str("\t\t\t\t\t\t}\n");
                    wave_portion.push_str("\t\t\t\t\t}\n");
                }else{
                    parse_bot_attributes(bot, &mission, &mut wave_portion, &mut context);
                    if !bot.behavior.is_empty(){
                        wave_portion.push_str(&format!("\t\t\t\t\tBehaviorModifiers\t{}\n", bot.behavior));
                    }
                }

                wave_portion.push_str("\t\t\t\t}\n");
                wave_portion.push_str("\t\t\t}\n");
                wave_portion.push_str("\t\t}\n");
            }
            last_bot = bot_id;
            if wavespawn.spawn_tank {
                wave_portion.push_str("\t\tWaveSpawn\n\t\t{\n");
                wave_portion.push_str(&format!("\t\t\tName\t\"w{}_b{}_tank\"\n", i, bot_id));
                if last_bot != 0 {
                    wave_portion.push_str(&format!("\t\t\tWaitForAllDead\t\"w{}_b{}\"\n", i, last_bot));
                }

                wave_portion.push_str(&format!("\t\t\tTotalCount\t1\n"));
                wave_portion.push_str(&format!("\t\t\tMaxActive\t1\n"));
                wave_portion.push_str(&format!("\t\t\tSpawnCount\t1\n"));
                wave_portion.push_str(&format!("\t\t\tWaitBeforeStarting\t0\n"));
                wave_portion.push_str(&format!("\t\t\tWaitBetweenSpawns\t5\n"));
                wave_portion.push_str(&format!("\t\t\tWhere\t{}\n", "spawnbot"));
                wave_portion.push_str(&format!("\t\t\tTotalCurrency\t{:.0}\n", 50.0 as f64 / total_weight as f64 * money_for_wave as f64 ));

                wave_portion.push_str("\t\t\tSquad\n\t\t\t{\n\t\t\t\tTank\n\t\t\t\t{\n");

                let eval_health = eval_float_with_context_mut(&mission.tank_health_formula, &mut context).unwrap();
                let speed_mod = rand::thread_rng().gen_range(0.3..2.0);

                wave_portion.push_str(&format!("\t\t\t\t\tHealth\t{:.0}\n",eval_health/speed_mod));
                wave_portion.push_str(&format!("\t\t\t\t\tSpeed\t{:.0}\n",75.0*speed_mod));

                wave_portion.push_str("\t\t\t\t\tOnKilledOutput\n\t\t\t\t\t{\n");
                wave_portion.push_str("\t\t\t\t\t\tTarget\tboss_dead_relay\n");
                wave_portion.push_str("\t\t\t\t\t\tAction\tTrigger\n\t\t\t\t\t}\n");

                wave_portion.push_str("\t\t\t\t\tOnBombDroppedOutput\n\t\t\t\t\t{\n");
                wave_portion.push_str("\t\t\t\t\t\tTarget\tboss_deploy_relay\n");
                wave_portion.push_str("\t\t\t\t\t\tAction\tTrigger\n\t\t\t\t\t}\n");

                wave_portion.push_str("\t\t\t\t}\n");
                wave_portion.push_str("\t\t\t}\n");
                wave_portion.push_str("\t\t}\n");
            }
        }
        wave_portion.push_str("\t}\n");
        wave_portion
    }).collect();
    
    pop_file.push_str(&generation);
    pop_file.push_str("}");
    output_file.write_all(pop_file.as_bytes())?;

    println!("Finished in {:?}", beginning.elapsed());
    Ok(())
}

fn parse_bot_attributes(bot:&Bot, mission:&Mission, wave_portion:&mut String, context:&mut HashMapContext){
    for item in &bot.weapons{
        wave_portion.push_str(&format!("\t\t\t\t\tItem\t\"{}\"\n", item));
    }
    for tag in &bot.tags{
        wave_portion.push_str(&format!("\t\t\t\t\tTag\t\"{}\"\n", tag));
    }

    if !bot.attributes.is_empty() || !mission.global_attributes.is_empty(){
        wave_portion.push_str("\t\t\t\t\tCharacterAttributes\n\t\t\t\t\t{\n");
        //bot specific attributes
        for attribute in &bot.attributes{
            let evaluation = match eval_with_context_mut(&attribute[1], context).unwrap(){
                Float(val) => val,
                Int(val) => val as f64,
                _ => panic!("Error while parsing {}", attribute[1])
            };
            wave_portion.push_str(&format!("\t\t\t\t\t\t\"{}\"\t{}\n", attribute[0], evaluation));
        }
        //mission global attributes
        for attribute in &mission.global_attributes{
            let evaluation = match eval_with_context_mut(&attribute[1], context).unwrap(){
                Float(val) => val,
                Int(val) => val as f64,
                _ => panic!("Error while parsing {}", attribute[1])
            };
            wave_portion.push_str(&format!("\t\t\t\t\t\t\"{}\"\t{}\n", attribute[0], evaluation));
        }
        wave_portion.push_str("\t\t\t\t\t}\n");
    }

    if !bot.weapon_attributes.is_empty() && !bot.weapons.is_empty(){
        wave_portion.push_str("\t\t\t\t\tItemAttributes\n\t\t\t\t\t{\n");
        //primary specific attributes
        wave_portion.push_str(&format!("\t\t\t\t\t\tItemName\t\"{}\"\n", bot.weapons.get(0).unwrap()));
        for attribute in &bot.weapon_attributes{
            let evaluation = match eval_with_context_mut(&attribute[1], context).unwrap(){
                Float(val) => val,
                Int(val) => val as f64,
                _ => panic!("Error while parsing {}", attribute[1])
            };
            wave_portion.push_str(&format!("\t\t\t\t\t\t\"{}\"\t{}\n", attribute[0], evaluation));
        }
        wave_portion.push_str("\t\t\t\t\t}\n");
    }
}
fn give_gatebot_cap_name(s: &str) -> String {
    match s {
        "scout" => "MvM GateBot Light Scout".to_string(),
        "soldier" => "MvM GateBot Light Soldier".to_string(),
        "pyro" => "MvM GateBot Light Pyro".to_string(),
        "demoman" => "MvM GateBot Light Demoman".to_string(),
        "heavyweapons" => "MvM GateBot Light Heavy".to_string(),
        "engineer" => "MvM GateBot Light Engineer".to_string(),
        "medic" => "MvM GateBot Light Medic".to_string(),
        "sniper" => "MvM GateBot Light Sniper".to_string(),
        "spy" | _ => "MvM GateBot Light Spy".to_string(),
    }
}