use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_program,
    program::invoke,
    instruction::Instruction,
    program,
};
use std::collections::{HashMap, VecDeque};

// Trading Pair Struct
#[derive(BorshDeserialize, BorshSerialize, Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct TradingPair{
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
}

// Order Struct
#[derive(BorshDeserialize, BorshSerialize, Debug, Clone, Default)]
pub struct Order{
    pub id: u32,
    pub trading_pair: TradingPair,
    pub order_type: String,    // "Limit", "Market", etc.
    pub side: String,    // "Buy" or "Sell"
    pub price: u64,
    pub amount: u64,
    pub filled_amount: u64,
    pub timestamp: u64,
    pub status: String, // Open, Filled, Cancelled
    pub dex_order_id: Option<Vec<u8>>,
    // Add other order details as needed
}

// Position Struct
#[derive(BorshDeserialize, BorshSerialize, Debug, Clone, Default)]
pub struct Position {
  pub trading_pair: TradingPair,
  pub base_amount: u64, // Amount of the base currency held
  pub quote_amount: u64, // Amount of the quote currency held
}

// Agent Configuration (DeFi Bot)
#[derive(BorshDeserialize, BorshSerialize, Debug, Clone)]
pub struct AgentConfig {
    pub owner: Pubkey,
    pub description: String,
    pub dex_program_id: Pubkey, // DEX program ID to interact with
    pub trading_pair: TradingPair,
    pub strategy_type: String, // Example: "SMA Crossover", "RSI Strategy"
    pub risk_parameters: RiskParameters,
     // Add more DeFi bot specific settings
}

// Risk Management parameters
#[derive(BorshDeserialize, BorshSerialize, Debug, Clone)]
pub struct RiskParameters{
    pub take_profit_percentage: f64, // Example 0.05 for 5%
    pub stop_loss_percentage: f64, // Example 0.03 for 3%
}

// Agent Instance Structure
#[derive(BorshDeserialize, BorshSerialize, Debug, Clone)]
pub struct AgentInstance {
    pub agent_id: u32,
    pub status: u8,         // 0: created, 1: running, 2: completed, 3: error
    pub start_time: u64,
    pub current_position: Position,
}

// Program State
#[derive(BorshDeserialize, BorshSerialize, Debug, Default)]
pub struct ProgramState {
    pub next_agent_id: u32,
    pub next_order_id: u32,
    pub agent_configs: Vec<AgentConfig>,
     pub agent_instances: Vec<AgentInstance>,
    pub open_orders: HashMap<u32, Order>, // Order id to order
    pub order_history: HashMap<TradingPair, Vec<Order>>,
    pub positions: HashMap<TradingPair, Position>, // Map trading pair to position
     pub last_analysis_time: u64,
}

// Define Instruction Enum
#[derive(BorshDeserialize, BorshSerialize, Debug, Clone)]
pub enum AgentInstruction {
    CreateAgent(AgentConfig),
    CreateAgentInstance { agent_id: u32 },
    UpdateAgentInstanceStatus { agent_id: u32, instance_id: u32, status: u8 },
     CreateOrder {agent_id: u32, trading_pair: TradingPair, order_type: String, side: String, price: u64, amount: u64},
     CancelOrder {order_id: u32},
     UpdateOrderStatus {order_id: u32, status: String, filled_amount: u64, dex_order_id: Option<Vec<u8>>},
    AnalyzeMarketAndTrade { agent_id: u32 }
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
         AgentInstruction::CreateOrder {agent_id, trading_pair, order_type, side, price, amount} => {
              msg!("Creating a new order");
              create_order(&mut program_state, agent_id, trading_pair, order_type, side, price, amount, state_account, program_id, accounts)?;
         }
         AgentInstruction::CancelOrder{order_id} => {
             msg!("Cancelling an order");
             cancel_order(&mut program_state, order_id, state_account, program_id, accounts)?;
         }
         AgentInstruction::UpdateOrderStatus{order_id, status, filled_amount, dex_order_id} => {
             msg!("Updating an order");
             update_order_status(&mut program_state, order_id, status, filled_amount, dex_order_id, state_account)?;
        }
        AgentInstruction::AnalyzeMarketAndTrade {agent_id} => {
            msg!("Analyzing market data and trading");
            analyze_market_and_trade(&mut program_state, agent_id, state_account, program_id, accounts)?;
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
   _state_account: &AccountInfo,
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
        current_position: Position::default(),
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
    _state_account: &AccountInfo,
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

fn create_order(
    program_state: &mut ProgramState,
    agent_id: u32,
    trading_pair: TradingPair,
    order_type: String,
    side: String,
    price: u64,
    amount: u64,
   _state_account: &AccountInfo,
    program_id: &Pubkey,
    accounts: &[AccountInfo]
)-> ProgramResult{
     // Check if agent exists
    if program_state.agent_configs.len() <= agent_id as usize {
        msg!("Agent not found");
        return Err(ProgramError::InvalidArgument);
    }

    let agent_config = &program_state.agent_configs[agent_id as usize];

    // Create new order
      let order = Order {
        id: program_state.next_order_id,
        trading_pair,
        order_type,
        side,
        price,
        amount,
        filled_amount: 0,
        timestamp: solana_program::sysvar::clock::Clock::get().unwrap().unix_timestamp as u64,
        status: "Open".to_string(),
         dex_order_id: None,
     };

      // Perform CPI to DEX (example)
        let dex_cpi_result = execute_dex_cpi(agent_config, &order, program_id, accounts);
        let dex_order_id = match dex_cpi_result {
           Ok(result) => Some(result),
           Err(_err) => None
        };

      let mut order = order.clone();
       order.dex_order_id = dex_order_id;

     // Store new order
      program_state.open_orders.insert(order.id, order.clone());

     // Record in history
      let order_history = program_state.order_history.entry(trading_pair).or_insert_with(Vec::new);
        order_history.push(order);

     program_state.next_order_id += 1;

     msg!("Order created with ID: {}", order.id);
    Ok(())
}


fn cancel_order(
    program_state: &mut ProgramState,
    order_id: u32,
    _state_account: &AccountInfo,
    program_id: &Pubkey,
     accounts: &[AccountInfo]
) -> ProgramResult {

      // Check if the order exists
     if !program_state.open_orders.contains_key(&order_id) {
        msg!("Order not found");
        return Err(ProgramError::InvalidArgument);
    }

     let order = program_state.open_orders.get_mut(&order_id).unwrap();
    
     // Check if the order is open or already filled
       if order.status != "Open" {
         msg!("Cannot cancel a non-open order.");
          return Err(ProgramError::InvalidArgument);
       }
        
       let agent_config = program_state.agent_configs.iter().find(|x| x.trading_pair == order.trading_pair).unwrap();

      //Perform CPI to DEX (Example)
     let _ = cancel_dex_cpi(agent_config, &order, program_id, accounts);

       // Update Order Status
      order.status = "Cancelled".to_string();

     msg!("Order cancelled with ID: {}", order_id);
     Ok(())
}

fn update_order_status(
    program_state: &mut ProgramState,
    order_id: u32,
    status: String,
    filled_amount: u64,
    dex_order_id: Option<Vec<u8>>,
     _state_account: &AccountInfo,
) -> ProgramResult {
     // Check if the order exists
     if !program_state.open_orders.contains_key(&order_id) {
        msg!("Order not found");
        return Err(ProgramError::InvalidArgument);
    }

      let order = program_state.open_orders.get_mut(&order_id).unwrap();

       // Update Order Status
      order.status = status.clone();
       order.filled_amount = filled_amount;
        if dex_order_id.is_some() {
            order.dex_order_id = dex_order_id;
         }

        // if status is filled, then remove it from open orders
        if status == "Filled" {
              program_state.open_orders.remove(&order_id);
             // Update Position
           update_position(program_state, order);
        }

     msg!("Order status updated to {} with ID: {}", status, order_id);
    Ok(())
}


fn analyze_market_and_trade(
    program_state: &mut ProgramState,
     agent_id: u32,
    _state_account: &AccountInfo,
    program_id: &Pubkey,
    accounts: &[AccountInfo]
) -> ProgramResult {
      // Check if agent exists
    if program_state.agent_configs.len() <= agent_id as usize {
        msg!("Agent not found");
        return Err(ProgramError::InvalidArgument);
    }

    let agent_config = &program_state.agent_configs[agent_id as usize];

    // Example Logic - fetch current price and create a new order based on the price
    let current_price = fetch_current_price();

      //Fetch current position for the given trading pair
      let position = program_state.positions.get(&agent_config.trading_pair);
    
    // Check current price against position and risk parameters.
      if let Some(position) = position {
           let new_order = check_risk_parameters(agent_config, position, current_price);
              if let Some(order) = new_order{
                   msg!("Creating new order based on risk parameters");
                    create_order(program_state, agent_id, agent_config.trading_pair.clone(), order.order_type, order.side, order.price, order.amount, _state_account, program_id, accounts)?;
                }
      }else{
        // If there is no position create a buy order at current price to initialize position
       create_order(program_state, agent_id, agent_config.trading_pair.clone(), "Market".to_string(), "Buy".to_string(), current_price, 1, _state_account, program_id, accounts)?;
      }
       program_state.last_analysis_time =  solana_program::sysvar::clock::Clock::get().unwrap().unix_timestamp as u64;

    Ok(())
}

// Example DEX CPI (Cross-Program Invocation)
fn execute_dex_cpi(_agent_config: &AgentConfig, order: &Order, _program_id: &Pubkey, _accounts: &[AccountInfo]) -> Result<Vec<u8>, ProgramError> {
   msg!("Executing DEX CPI");

   // Build DEX instruction using the agent_config and order.
   // You would use an instruction to interact with another program
  
  //Dummy order_id for the example
  let dex_order_id: Vec<u8> = vec![1, 2, 3, 4];
  Ok(dex_order_id)
}


// Example cancel CPI to DEX (Cross-Program Invocation)
fn cancel_dex_cpi(_agent_config: &AgentConfig, order: &Order, _program_id: &Pubkey, _accounts: &[AccountInfo]) -> Result<(), ProgramError> {
  msg!("Cancelling DEX CPI");
   // Build DEX instruction to cancel order using the order.
   // You would use an instruction to interact with another program
  Ok(())
}

// Example function to fetch current price (replace with real data feed)
fn fetch_current_price() -> u64{
   10 // Example price data
}

fn check_risk_parameters(config: &AgentConfig, position: &Position, current_price: u64) -> Option<Order>{
       let take_profit_percentage = config.risk_parameters.take_profit_percentage;
       let stop_loss_percentage = config.risk_parameters.stop_loss_percentage;

       //get the amount of the base currency
       let base_amount = position.base_amount as f64;
    
        if base_amount == 0.0 {
            return None
        }
      
    let entry_price = (position.quote_amount as f64 / position.base_amount as f64) as u64;
    let price_difference = current_price as f64 - entry_price as f64;
    let price_difference_percentage = price_difference / entry_price as f64;
    
     if price_difference_percentage >= take_profit_percentage {
           return Some(Order {
                id: 0, // Dummy value as order id will be generated later
                trading_pair: config.trading_pair.clone(),
                order_type: "Market".to_string(),
                side: "Sell".to_string(),
                price: current_price,
                amount: position.base_amount,
                filled_amount: 0,
                timestamp: solana_program::sysvar::clock::Clock::get().unwrap().unix_timestamp as u64,
                status: "Open".to_string(),
                 dex_order_id: None,
           });
     }
    
    if price_difference_percentage <= -stop_loss_percentage {
          return Some(Order {
                id: 0, // Dummy value as order id will be generated later
                trading_pair: config.trading_pair.clone(),
                order_type: "Market".to_string(),
                side: "Sell".to_string(),
                price: current_price,
                amount: position.base_amount,
                filled_amount: 0,
                timestamp: solana_program::sysvar::clock::Clock::get().unwrap().unix_timestamp as u64,
                status: "Open".to_string(),
                 dex_order_id: None,
           });
    }
   None
}


fn update_position(program_state: &mut ProgramState, order: &Order){
      let position = program_state.positions.entry(order.trading_pair.clone()).or_insert(Position{
          trading_pair: order.trading_pair.clone(),
          base_amount: 0,
          quote_amount: 0,
      });
    // Update position based on the order execution
     if order.side == "Buy" {
            position.base_amount += order.amount;
            position.quote_amount += order.amount * order.price;
     }

    if order.side == "Sell"{
            position.base_amount -= order.amount;
            position.quote_amount -= order.amount * order.price;
    }
}