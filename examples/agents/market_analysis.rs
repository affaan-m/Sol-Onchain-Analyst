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
use std::collections::HashMap;

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

// Agent Configuration
#[derive(BorshDeserialize, BorshSerialize, Debug, Clone)]
pub struct AgentConfig {
    pub owner: Pubkey,      // Owner of this agent
    pub description: String,  // Task description
    pub trading_pair: String, // Example: "SOL/USDC"
    pub timeframes: Vec<TimeFrame>,
    pub indicators: Vec<String>, // Example: ["SMA_20", "RSI_14"]
}

// Agent Instance Structure
#[derive(BorshDeserialize, BorshSerialize, Debug, Clone)]
pub struct AgentInstance {
    pub agent_id: u32,        // ID of the agent config
    pub status: u8,         // 0: created, 1: running, 2: completed, 3: error
    pub start_time: u64,
}


// Program State (Account Data)
#[derive(BorshDeserialize, BorshSerialize, Debug, Default)]
pub struct ProgramState {
    pub next_agent_id: u32,        // Counter to assign unique ids for agents
    pub agent_configs: Vec<AgentConfig>,
    pub agent_instances: Vec<AgentInstance>,
    // Mapping of (TradingPair, TimeFrame, Timestamp) -> Market Data
    pub market_data: HashMap<(String, TimeFrame, u64), MarketData>,
}


// Define Instruction Enum
#[derive(BorshDeserialize, BorshSerialize, Debug, Clone)]
pub enum AgentInstruction {
    CreateAgent(AgentConfig),
    CreateAgentInstance { agent_id: u32 },
    UpdateAgentInstanceStatus { agent_id: u32, instance_id: u32, status: u8 },
    UpdateMarketData{trading_pair: String, timeframe: TimeFrame, market_data: MarketData},
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