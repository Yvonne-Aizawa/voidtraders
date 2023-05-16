use chrono::{DateTime, Utc};
use dateparser::parse;
use spacedust::apis::agents_api::get_my_agent;
use spacedust::apis::default_api::register;
use spacedust::apis::fleet_api::{
    dock_ship, extract_resources, get_my_ships, navigate_ship, orbit_ship, refuel_ship, sell_cargo,
};
use spacedust::apis::systems_api::get_system_waypoints;
use spacedust::models::register_request::{Faction, RegisterRequest};
use spacedust::models::{
    ExtractResources201Response, GetMyAgent200Response, NavigateShipRequest, ShipCargoItem,
    ShipNavStatus,
};
mod parsers;
use crate::parsers::{
    extract_resources_response_parser, get_agent_response_parser, sell_cargo_response_parser,
    travel_request_response_parser, ExtractResourcesErrorParsed, GetMyAgentErrorParsed,
};

use tokio::{
    task,
    time::{interval, Duration},
};

#[tokio::main]
async fn main() {
    let mut interval = interval(Duration::from_secs(10));
    let mut count = 1;
    loop {
        if count > 3 {
            count = 1
        }
        interval.tick().await;
        task::spawn(spacetraders(count));
        count += 1;
    }
}

async fn spacetraders(count: i32) {
    let api_key_str = void_config::get_config_key("spacetraders", "token");
    let configuration = create_configuration(api_key_str.to_owned());
    let agent = show_agent_details(&configuration).await;
    match agent {
        Ok(_) => {
            run_logic(&configuration, count).await;
        }
        Err(err) => {
            println!("{}", err.code);
            if err.code == 4104 {
                //create new account since new old one is ded\
                let mut conf = spacedust::apis::configuration::Configuration::new();

                conf.base_path = void_config::get_config_key("spacetraders", "url");
                println!("Creating new account");
                let register_request = RegisterRequest {
                    faction: Faction::Void,
                    symbol: "yvonne-aizawa".to_string(),
                };
                let register_response = register(&conf, Some(register_request)).await.unwrap();
                void_config::set_config_key("spacetraders", "token", &register_response.data.token);
                dbg!(register_response);
            }
            if err.code == 4103 {
                dbg!(err);
            }
        }
    }
}
pub async fn run_logic(configuration: &spacedust::apis::configuration::Configuration, count: i32) {
    let ships = get_my_ships(configuration, Some(count), Some(5))
        .await
        .unwrap();

    for ship in ships.data.iter() {
        println!("{} at {}", ship.symbol, ship.nav.waypoint_symbol);
        print_ship_info(ship);
        if ship.nav.status.to_string() == "IN_ORBIT" {
            //dock ship
            drop(dock_ship(configuration, &ship.symbol, 0.0).await);
        }
        if ship.nav.status.to_string() == "IN_TRANSIT" {
            //do nothing
        }
        if ship.nav.status.to_string() == "DOCKED" {
            auto_sell(configuration, ship).await;
            let mine_result = auto_mine(configuration, ship).await;
            if let Err(err) = mine_result {
                if err.code == 4228 {
                    navigate_to(configuration, ship, "X1-DC54-89945X").await
                } else if err.code == 4205 {
                    let planet = find_sutable_mine_planet(configuration, ship).await;
                    navigate_to(configuration, ship, &planet).await
                }
            }
        }
    }
}
pub async fn find_sutable_mine_planet(
    configuration: &spacedust::apis::configuration::Configuration,
    ship: &spacedust::models::Ship,
) -> String {
    let waypoints =
        get_system_waypoints(configuration, &ship.nav.system_symbol, Some(1), Some(10)).await;
    //get a random planet
    //loop though the waypoints
    for waypoint in waypoints.unwrap().data.iter() {
        //loop though the traits
        for trait_ in waypoint.traits.iter() {
            if trait_.symbol == spacedust::models::waypoint_trait::Symbol::PreciousMetalDeposits {
                return waypoint.symbol.to_string();
            }
        }
    }
    "X1-DC54-89945X".to_string()
}
pub async fn navigate_to(
    configuration: &spacedust::apis::configuration::Configuration,
    ship: &spacedust::models::Ship,
    waypoint: &str,
) {
    let nav_ship_request = NavigateShipRequest {
        waypoint_symbol: waypoint.to_string(),
    };
    
    let parsed_res = travel_request_response_parser(
        navigate_ship(configuration, &ship.symbol, Some(nav_ship_request)).await,
    );
    match parsed_res {
        Ok(res) => {
            println!(
                "Traveling to waypoint {}",
                res.data.nav.route.destination.symbol
            );
            // wait for travel to finish
        }
        Err(err) => {
            println!("unable to travel to waypoint {}: {}", err.message, err.code);
            if err.code == 4203 {
                // no fuel
                println!("refueling ship");
                drop(refuel_ship(configuration, &ship.symbol, 0).await);
            } else if err.code == 4204 {
                // ship is docked. undock the ship
                drop(orbit_ship(configuration, &ship.symbol, 0).await);
            }
        }
    }
}
async fn auto_sell(
    configuration: &spacedust::apis::configuration::Configuration,
    ship: &spacedust::models::Ship,
) {
    // loop though cargo
    for cargo in ship.cargo.inventory.iter() {
        //if it is ALUMINUM_ORE skip we need it for the contract
        // println!("cargo: {}:{}", cargo.symbol, cargo.units);
        if cargo.symbol != "ANTIMATTER" {
            let request = spacedust::models::SellCargoRequest {
                symbol: cargo.symbol.clone(),
                units: cargo.units,
            };
            let result = sell_cargo(configuration, &ship.symbol, Some(request)).await;
            let parsed_res = sell_cargo_response_parser(result);
            match parsed_res {
                Ok(res) => {
                    println!(
                        "Selling cargo {}x{}",
                        res.data.transaction.trade_symbol, res.data.transaction.units
                    );
                }
                Err(err) => {
                    println!("error extracting resources: {}", err.message);
                }
            }
        }
    }
}

fn print_ship_info(ship: &spacedust::models::Ship) {
    match ship.nav.status {
        ShipNavStatus::Docked => {
            // ship is docked so show where it is docked
            println!(
                "ship {} is docked at {} {}/{}",
                ship.symbol, ship.nav.waypoint_symbol, ship.cargo.units, ship.cargo.capacity
            );
        }
        ShipNavStatus::InOrbit => {
            //ship is in orbit show where it is in orbit
            println!(
                "ship {} is in orbit at {}",
                ship.symbol, ship.nav.waypoint_symbol
            );
        }
        ShipNavStatus::InTransit => {
            // ship is in transit show destination and time to arrival
            println!(
                "ship: {} is traveling from {} to {} and will arive in {}",
                ship.symbol,
                ship.nav.route.departure.symbol,
                ship.nav.route.destination.symbol,
                get_time_till_arival(&ship.nav.route.arrival)
            );
        }
    };
}

async fn auto_mine(
    configuration: &spacedust::apis::configuration::Configuration,
    ship: &spacedust::models::Ship,
) -> Result<ExtractResources201Response, ExtractResourcesErrorParsed> {
    let request = spacedust::models::ExtractResourcesRequest { survey: None };

    let res = extract_resources(configuration, &ship.symbol, Some(request)).await;
    // print_type_of(&res);
    let parsed_res = extract_resources_response_parser(res);
    match parsed_res {
        Ok(res) => {
            println!(
                "extracting resources: {}x{:?}",
                res.data.extraction.r#yield.symbol, res.data.extraction.r#yield.units
            );
            Ok(res)
        }
        Err(err) => {
            println!(
                "error extracting resources: {},  code: {}",
                err.message, err.code
            );
            Err(err)
        }
    }
}

fn create_configuration(key: String) -> spacedust::apis::configuration::Configuration {
    let mut configuration = spacedust::apis::configuration::Configuration::new();

    configuration.bearer_access_token = Some(key);
    configuration.base_path = void_config::get_config_key("spacetraders", "url");
    configuration
}

fn get_time_till_arival(arrival: &str) -> i64 {
    // 2023-05-11T17:51:34.699Z example string
    let parsed_date = parse(arrival).unwrap();
    let date_time: DateTime<Utc> = parsed_date;
    let current_time = Utc::now();
    let time_since_date = current_time - date_time;
    if time_since_date.num_seconds() >= 0 {
        return 0;
    }

    time_since_date.num_seconds().abs()
}
fn print_type_of<T>(_: &T) {
    println!("the type is: {}", std::any::type_name::<T>())
}
fn get_cargo_amount(cargo: &Vec<ShipCargoItem>, symbol: &str) -> i32 {
    let mut total = 0;
    for item in cargo {
        if item.symbol == symbol {
            total += item.units;
        }
    }
    total
}
async fn show_agent_details(
    configuration: &spacedust::apis::configuration::Configuration,
) -> Result<GetMyAgent200Response, GetMyAgentErrorParsed> {
    let parsed_agent = get_agent_response_parser(get_my_agent(configuration).await);
    match parsed_agent {
        Ok(agent) => {
            println!(
                "agent details: {} credits: {}",
                agent.data.symbol, agent.data.credits
            );
            Ok(agent)
        }
        Err(err) => Err(err),
    }
}
