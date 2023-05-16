use spacedust::apis::agents_api::GetMyAgentError;
use spacedust::apis::fleet_api::ExtractResourcesError;
use spacedust::apis::fleet_api::NavigateShipError;
use spacedust::apis::fleet_api::SellCargoError;
use spacedust::apis::Error::ResponseError;
use spacedust::models::ExtractResources201Response;
use spacedust::models::GetMyAgent200Response;
use spacedust::models::NavigateShip200Response;
use spacedust::models::SellCargo201Response;

#[derive(Debug)]
pub struct SellCargoErrorParsed {
    pub code: i32,
    pub message: String,
}
pub fn sell_cargo_response_parser(
    response: core::result::Result<
        SellCargo201Response,
        spacedust::apis::Error<spacedust::apis::fleet_api::SellCargoError>,
    >,
) -> Result<SellCargo201Response, SellCargoErrorParsed> {
    match response {
        Ok(res) => Ok(res),
        Err(err) => match err {
            ResponseError(err) => {
                if let Some(SellCargoError::UnknownValue(err)) = err.entity {
                    Err(SellCargoErrorParsed {
                        code: err["error"]["code"].to_string().parse().unwrap(),
                        message: err["error"]["message"].to_string(),
                    })
                } else {
                    Err(SellCargoErrorParsed {
                        code: 0,
                        message: "".to_string(),
                    })
                }
            }
            _ => todo!(),
        },
    }
}
#[derive(Debug)]
pub struct ExtractResourcesErrorParsed {
    pub code: i32,
    pub message: String,
}

// list of codes
// cooldown 4000
// invalid planet 4205
// full inventory 4228
pub fn extract_resources_response_parser(
    response: core::result::Result<
        ExtractResources201Response,
        spacedust::apis::Error<spacedust::apis::fleet_api::ExtractResourcesError>,
    >,
) -> Result<ExtractResources201Response, ExtractResourcesErrorParsed> {
    match response {
        Ok(res) => Ok(res),
        Err(err) => match err {
            ResponseError(err) => {
                if let Some(ExtractResourcesError::UnknownValue(err)) = err.entity {
                    Err(ExtractResourcesErrorParsed {
                        code: err["error"]["code"].to_string().parse().unwrap(),
                        message: err["error"]["message"].to_string(),
                    })
                } else {
                    Err(ExtractResourcesErrorParsed {
                        code: 0,
                        message: "".to_string(),
                    })
                }
            }
            _ => todo!(),
        },
    }
}
// codes
// no fuel 4203
#[derive(Debug)]
pub struct NavigateShipErrorParsed {
    pub code: i32,
    pub message: String,
}
pub fn travel_request_response_parser(
    response: Result<NavigateShip200Response, spacedust::apis::Error<NavigateShipError>>,
) -> Result<NavigateShip200Response, NavigateShipErrorParsed> {
    match response {
        Ok(res) => Ok(res),
        Err(err) => match err {
            ResponseError(err) => {
                if let Some(NavigateShipError::UnknownValue(err)) = err.entity {
                    Err(NavigateShipErrorParsed {
                        code: err["error"]["code"].to_string().parse().unwrap(),
                        message: err["error"]["message"].to_string(),
                    })
                } else {
                    Err(NavigateShipErrorParsed {
                        code: 0,
                        message: "".to_string(),
                    })
                }
            }
            _ => todo!(),
        },
    }
}
#[derive(Debug)]
pub struct GetMyAgentErrorParsed {
    pub code: i32,
    pub message: String,
}
pub fn get_agent_response_parser(
    response: Result<GetMyAgent200Response, spacedust::apis::Error<GetMyAgentError>>,
) -> Result<GetMyAgent200Response, GetMyAgentErrorParsed> {
    match response {
        Ok(res) => Ok(res),
        Err(err) => match err {
            ResponseError(err) => {
                if let Some(GetMyAgentError::UnknownValue(err)) = err.entity {
                    Err(GetMyAgentErrorParsed {
                        code: err["error"]["code"].to_string().parse().unwrap(),
                        message: err["error"]["message"].to_string(),
                    })
                } else {
                    Err(GetMyAgentErrorParsed {
                        code: 0,
                        message: "".to_string(),
                    })
                }
            }
            _err => {
                dbg!(_err);
                todo!();
            }
        },
    }
}
