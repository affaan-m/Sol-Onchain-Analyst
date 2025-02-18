use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_program,
};
use std::collections::{HashMap, VecDeque};


// Market Data Structs
#[derive(BorshDeserialize, BorshSerialize, Debug, Clone, Default)]
pub struct MarketData {
  pub timestamp: u64,
  pub open: f64,
  pub high: f64,
  pub low: f64,
  pub close: f64,
  pub volume: f64,
}

// TimeFrame (enum)
#[derive(BorshDeserialize, BorshSerialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum TimeFrame {
    OneMinute,
    FiveMinutes,
    FifteenMinutes,
    OneHour,
    FourHours,
    OneDay,
}

// Opportunity Struct
#[derive(BorshDeserialize, BorshSerialize, Debug, Clone, Default)]
pub struct Opportunity {
  pub trading_pair: String,
  pub timeframe: TimeFrame,
  pub signal_type: String,   // Example "SMA Crossover"
  pub timestamp: u64,
  pub additional_info: String,
}

// Agent Configuration
#[derive(BorshDeserialize, BorshSerialize, Debug, Clone)]
pub struct AgentConfig {
    pub owner: Pubkey,          // Owner of this agent
    pub description: String,     // Task description
    pub trading_pair: String,    // Example: "SOL/USDC"
    pub timeframes: Vec<TimeFrame>,
    pub indicators: Vec<String>,   // Example: ["SMA_20", "RSI_14"]
    pub opportunity_criteria: OpportunityCriteria,
}

// Opportunity Criteria (Example)
#[derive(BorshDeserialize, BorshSerialize, Debug, Clone)]
pub struct OpportunityCriteria{
    pub indicator_condition: String,  // Example: "SMA_20_CROSS_UP_SMA_50"
    // Add other criteria
}

// Agent Instance Structure
#[derive(BorshDeserialize, BorshSerialize, Debug, Clone)]
pub struct AgentInstance {
    pub agent_id: u32,        // ID of the agent config
    pub status: u8,         // 0: created, 1: running, 2: completed, 3: error
    pub start_time: u64,
    pub triggered_opportunity: Option<Opportunity>,
}

// Program State (Account Data)
#[derive(BorshDeserialize, BorshSerialize, Debug, Default)]
pub struct ProgramState {
    pub next_agent_id: u32,        // Counter to assign unique ids for agents
    pub agent_configs: Vec<AgentConfig>,
    pub agent_instances: Vec<AgentInstance>,
    pub market_data: HashMap<(String, TimeFrame, u64), MarketData>,
    pub opportunities: Vec<Opportunity>,
    pub last_analysis_time: u64,
}


// Define Instruction Enum
#[derive(BorshDeserialize, BorshSerialize, Debug, Clone)]
pub enum AgentInstruction {
    CreateAgent(AgentConfig),
    CreateAgentInstance { agent_id: u32 },
    UpdateAgentInstanceStatus { agent_id: u32, instance_id: u32, status: u8 },
    UpdateMarketData{trading_pair: String, timeframe: TimeFrame, market_data: MarketData},
    AnalyzeMarketOpportunities { agent_id: u32 },
}


// Entrypoint
entrypoint!(process_instruction);
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("AI Agent Program invoked!");

    let instruction = AgentInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

     let accounts_iter = &mut accounts.iter();
    let state_account = next_account_info(accounts_iter)?;

    if !state_account.is_writable {
        msg!("Program state account is not writeable");
        return Err(ProgramError::InvalidArgument);
    }
    
    // Load Program state (if available) or create a new one if not initialized
    let mut program_state = ProgramState::try_from_slice(&state_account.data.borrow())
         .unwrap_or_default();


    match instruction {
         AgentInstruction::CreateAgent(config) => {
            msg!("Creating agent config...");
            create_agent(&mut program_state, config, program_id, state_account)?;

        }
        AgentInstruction::CreateAgentInstance { agent_id } => {
            msg!("Creating agent instance...");
           create_agent_instance(&mut program_state, agent_id, state_account)?;
        }

        AgentInstruction::UpdateAgentInstanceStatus {agent_id, instance_id, status} => {
            msg!("Updating agent instance status...");
             update_agent_instance_status(&mut program_state, agent_id, instance_id, status, state_account)?;
       }
       AgentInstruction::UpdateMarketData{trading_pair, timeframe, market_data} => {
            msg!("Updating market data");
            update_market_data(&mut program_state, trading_pair, timeframe, market_data, state_account)?;
        }
       AgentInstruction::AnalyzeMarketOpportunities { agent_id } => {
            msg!("Analyzing market opportunities...");
            analyze_market_opportunities(&mut program_state, agent_id, state_account)?;
        }
    }

     // Serialize the program state back to the account
     program_state.serialize(&mut &mut state_account.data.borrow_mut()[..])?;

    Ok(())
}

// Instruction implementations
fn create_agent(
    program_state: &mut ProgramState,
    config: AgentConfig,
    program_id: &Pubkey,
     state_account: &AccountInfo,
) -> ProgramResult {

    // Check if the signer is the owner of program
     if state_account.owner != program_id {
        msg!("Incorrect owner for program");
        return Err(ProgramError::IncorrectProgramId);
    }
    
    let config_id = program_state.next_agent_id;
    program_state.agent_configs.push(config.clone());
    program_state.next_agent_id += 1;

     msg!("Created agent with ID: {}", config_id);

    Ok(())
}

fn create_agent_instance(
    program_state: &mut ProgramState,
    agent_id: u32,
   state_account: &AccountInfo,
) -> ProgramResult {

      // Check if agent exists
     if program_state.agent_configs.len() <= agent_id as usize {
        msg!("Agent not found");
        return Err(ProgramError::InvalidArgument);
    }

    let new_instance = AgentInstance {
        agent_id,
        status: 0, // Created status
        start_time: solana_program::sysvar::clock::Clock::get().unwrap().unix_timestamp as u64,
        triggered_opportunity: None,
    };

     program_state.agent_instances.push(new_instance);

     msg!("Created agent instance with agent ID: {}", agent_id);

    Ok(())
}

fn update_agent_instance_status(
    program_state: &mut ProgramState,
    agent_id: u32,
    instance_id: u32,
    status: u8,
    state_account: &AccountInfo,
) -> ProgramResult {
    if program_state.agent_instances.len() <= instance_id as usize {
        msg!("Agent instance not found");
        return Err(ProgramError::InvalidArgument);
    }

     let instance = program_state.agent_instances.get_mut(instance_id as usize).unwrap();
     if instance.agent_id != agent_id {
        msg!("Incorrect agent ID for the requested instance");
        return Err(ProgramError::InvalidArgument)
    }

     instance.status = status;
     msg!("Updated agent instance status to: {}", status);
     Ok(())
}

fn update_market_data(
     program_state: &mut ProgramState,
    trading_pair: String,
    timeframe: TimeFrame,
    market_data: MarketData,
     _state_account: &AccountInfo,
)->ProgramResult{

     program_state.market_data.insert((trading_pair, timeframe, market_data.timestamp), market_data);
    
    Ok(())
}

fn analyze_market_opportunities(
    program_state: &mut ProgramState,
    agent_id: u32,
    _state_account: &AccountInfo,
) -> ProgramResult {
    // Check if agent exists
    if program_state.agent_configs.len() <= agent_id as usize {
        msg!("Agent not found");
        return Err(ProgramError::InvalidArgument);
    }

     let config = &program_state.agent_configs[agent_id as usize];
   
    // Add the logic for identifying opportunities based on config
    let opportunities = identify_opportunities(config, &program_state.market_data);

    for opportunity in opportunities {
          program_state.opportunities.push(opportunity.clone());
           // Iterate through instances and trigger if applicable
            for instance in program_state.agent_instances.iter_mut() {
                 if instance.agent_id == agent_id && instance.status == 0 { // Created
                   msg!("Triggering instance {}", instance.agent_id);
                  instance.status = 1;
                  instance.triggered_opportunity = Some(opportunity.clone());
              }
         }
    }

      program_state.last_analysis_time =  solana_program::sysvar::clock::Clock::get().unwrap().unix_timestamp as u64;
     Ok(())
}

fn identify_opportunities(
  config: &AgentConfig, 
  market_data: &HashMap<(String, TimeFrame, u64), MarketData>,
) -> Vec<Opportunity> {
   let mut opportunities = Vec::new();
    for timeframe in &config.timeframes {
        // Get all the market data for the current trading_pair and timeframe
        let data = market_data.iter()
                                .filter(|((trading_pair, tf, _),_)| trading_pair == &config.trading_pair && tf == timeframe)
                                .map(|((_, _, timestamp), data)|(timestamp, data)).collect::<Vec<_>>();

          // Order by timestamp to ensure logic of the opportunity detection is correct
          let mut sorted_data = data.clone();
          sorted_data.sort_by(|(a, _), (b, _)| a.cmp(b));

           // Add opportunity identification logic based on the `indicator_condition`
           let opportunity = check_opportunity_condition(&sorted_data, config, timeframe);

          if let Some(opp) = opportunity {
                opportunities.push(opp);
          }
    }

    opportunities
}

// Example opportunity check - this will need to be extended based on your logic needs
fn check_opportunity_condition(sorted_data: &Vec<(&u64, &MarketData)>, config: &AgentConfig, timeframe: &TimeFrame) -> Option<Opportunity> {
       if sorted_data.len() < 2 {
            return None; // Not enough data to analyze
        }

        // Example Logic (Simple SMA Crossover)
        let condition_type = config.opportunity_criteria.indicator_condition.clone();
        if condition_type == "SMA_20_CROSS_UP_SMA_50"{
            let last_data = sorted_data.last().unwrap();
            let previous_data = sorted_data.get(sorted_data.len() - 2).unwrap();

              let sma_20 = calculate_simple_moving_average(&sorted_data, 20);
              let sma_50 = calculate_simple_moving_average(&sorted_data, 50);

               if sma_20.is_some() && sma_50.is_some() {
                     let current_sma_20 = sma_20.unwrap().1;
                     let current_sma_50 = sma_50.unwrap().1;
                     
                    let prev_sma_20 = calculate_simple_moving_average(&sorted_data[0..sorted_data.len() - 1].to_vec(), 20);
                    let prev_sma_50 = calculate_simple_moving_average(&sorted_data[0..sorted_data.len() - 1].to_vec(), 50);

                  if prev_sma_20.is_some() && prev_sma_50.is_some(){
                        let previous_sma_20 = prev_sma_20.unwrap().1;
                         let previous_sma_50 = prev_sma_50.unwrap().1;
                        if previous_sma_20 <= previous_sma_50 && current_sma_20 > current_sma_50 {
                             return Some(Opportunity {
                                    trading_pair: config.trading_pair.clone(),
                                    timeframe: timeframe.clone(),
                                    signal_type: "SMA Crossover".to_string(),
                                    timestamp: *last_data.0,
                                    additional_info: "SMA_20 crossing above SMA_50".to_string(),
                                });
                         }
                    }
                 }
        }

    None
}

// Example SMA calculation - this will need to be extended based on your logic needs
fn calculate_simple_moving_average(sorted_data: &Vec<(&u64, &MarketData)>, period: usize) -> Option<(&u64, f64)> {
    if sorted_data.len() < period {
        return None;
    }
    let end_index = sorted_data.len();
    let start_index = end_index - period;
    let subset = &sorted_data[start_index..end_index];

    let sum: f64 = subset.iter().map(|(_, data)| data.close).sum();

     Some((sorted_data.last().unwrap().0, sum / period as f64))
}