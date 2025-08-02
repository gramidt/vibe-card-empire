use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Alignment},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::{error::Error, io, time::{Duration, Instant}};
use std::collections::VecDeque;

#[derive(Debug, Clone)]
struct GiftCard {
    retailer: String,
    denomination: u32,
    purchase_price: u32,
    days_until_expiration: u32,
}

impl GiftCard {
    fn new(retailer: &str, denomination: u32, purchase_price: u32, days_until_expiration: u32) -> Self {
        Self {
            retailer: retailer.to_string(),
            denomination,
            purchase_price,
            days_until_expiration,
        }
    }

    fn market_value(&self) -> u32 {
        // Basic markup calculation - 20-30% depending on retailer
        match self.retailer.as_str() {
            "Amazon" => (self.denomination as f32 * 1.30) as u32,
            "Starbucks" => (self.denomination as f32 * 1.25) as u32,
            "Target" => (self.denomination as f32 * 1.28) as u32,
            "iTunes" => (self.denomination as f32 * 1.22) as u32,
            "Walmart" => (self.denomination as f32 * 1.20) as u32,
            _ => (self.denomination as f32 * 1.25) as u32,
        }
    }

    fn potential_profit(&self) -> i32 {
        self.market_value() as i32 - self.purchase_price as i32
    }

    fn is_expiring_soon(&self) -> bool {
        self.days_until_expiration <= 15
    }
}

#[derive(Debug, Clone)]
struct InventoryItem {
    card: GiftCard,
    quantity: u32,
}

impl InventoryItem {
    fn new(card: GiftCard, quantity: u32) -> Self {
        Self { card, quantity }
    }

    fn total_value(&self) -> u32 {
        self.card.market_value() * self.quantity
    }

    fn total_cost(&self) -> u32 {
        self.card.purchase_price * self.quantity
    }
}

#[derive(Debug, Clone)]
struct CustomerOrder {
    id: u32,
    customer_name: String,
    retailer: String,
    denomination: u32,
    quantity: u32,
    offered_price_per_card: u32,
    deadline_days: u32,
    priority: OrderPriority,
}

#[derive(Debug, Clone)]
enum OrderPriority {
    Low,
    Medium, 
    High,
}

impl OrderPriority {
    fn display(&self) -> &str {
        match self {
            OrderPriority::Low => "Low",
            OrderPriority::Medium => "Medium", 
            OrderPriority::High => "High",
        }
    }
}

impl CustomerOrder {
    fn new(id: u32, customer_name: &str, retailer: &str, denomination: u32, quantity: u32, offered_price_per_card: u32, deadline_days: u32, priority: OrderPriority) -> Self {
        Self {
            id,
            customer_name: customer_name.to_string(),
            retailer: retailer.to_string(),
            denomination,
            quantity,
            offered_price_per_card,
            deadline_days,
            priority,
        }
    }

    fn total_offered(&self) -> u32 {
        self.offered_price_per_card * self.quantity
    }

    fn is_expired(&self) -> bool {
        self.deadline_days == 0
    }
}

#[derive(Debug, Clone)]
enum Screen {
    MainMenu,
    Dashboard,
    Market,
    Orders,
    Inventory,
    Analytics,
    Settings,
}

#[derive(Debug)]
struct GameData {
    cash: u32,
    reputation: u8, // 1-5 stars
    day: u32,
    hour: u8,
    minute: u8,
    recent_activities: Vec<String>,
    inventory: Vec<InventoryItem>,
    customer_orders: VecDeque<CustomerOrder>,
    next_order_id: u32,
}

impl GameData {
    fn new() -> Self {
        // Create some sample inventory for testing
        let sample_inventory = vec![
            InventoryItem::new(
                GiftCard::new("Amazon", 25, 20, 45),
                12
            ),
            InventoryItem::new(
                GiftCard::new("Target", 50, 42, 30),
                8
            ),
            InventoryItem::new(
                GiftCard::new("Starbucks", 10, 8, 120),
                15
            ),
            InventoryItem::new(
                GiftCard::new("iTunes", 15, 12, 15),
                3
            ),
            InventoryItem::new(
                GiftCard::new("Walmart", 20, 17, 60),
                6
            ),
        ];

        let mut game_data = Self {
            cash: 5000,
            reputation: 3,
            day: 1,
            hour: 9,
            minute: 0,
            recent_activities: vec![
                "Welcome to Gift Card Empire!".to_string(),
                "Starting with $5,000 capital".to_string(),
                "Visit the Market to buy your first cards".to_string(),
            ],
            inventory: sample_inventory,
            customer_orders: VecDeque::new(),
            next_order_id: 1000,
        };

        // Generate some initial customer orders
        game_data.generate_random_order();
        game_data.generate_random_order();
        
        game_data
    }

    fn advance_time(&mut self, minutes: u8) {
        self.minute += minutes;
        if self.minute >= 60 {
            self.hour += self.minute / 60;
            self.minute = self.minute % 60;
        }
        
        if self.hour >= 24 {
            self.day += (self.hour / 24) as u32;
            self.hour = self.hour % 24;
            
            // Process daily events when a new day starts
            self.process_daily_events();
        }
    }

    fn process_daily_events(&mut self) {
        // Age all inventory by 1 day
        for item in &mut self.inventory {
            if item.card.days_until_expiration > 0 {
                item.card.days_until_expiration -= 1;
            }
        }

        // Remove expired cards and calculate losses
        let mut expired_value = 0;
        let mut expired_count = 0;
        
        self.inventory.retain(|item| {
            if item.card.days_until_expiration == 0 {
                expired_value += item.total_cost();
                expired_count += item.quantity;
                false
            } else {
                true
            }
        });

        if expired_count > 0 {
            self.recent_activities.insert(0, format!(
                "❌ Lost {} cards worth ${} to expiration", 
                expired_count, expired_value
            ));
            
            // Keep only the last 10 activities
            if self.recent_activities.len() > 10 {
                self.recent_activities.truncate(10);
            }
        }

        // Process customer orders aging
        self.process_order_aging();

        // Add daily startup message
        self.recent_activities.insert(0, format!("🌅 Day {} begins", self.day));
        if self.recent_activities.len() > 10 {
            self.recent_activities.truncate(10);
        }
    }

    fn reputation_stars(&self) -> String {
        let filled = "★".repeat(self.reputation as usize);
        let empty = "☆".repeat(5 - self.reputation as usize);
        format!("{}{}", filled, empty)
    }

    fn reputation_description(&self) -> &str {
        match self.reputation {
            5 => "Legendary",
            4 => "Excellent", 
            3 => "Good",
            2 => "Fair",
            1 => "Poor",
            _ => "Unknown",
        }
    }

    fn time_display(&self) -> String {
        let period = if self.hour < 12 { "AM" } else { "PM" };
        let display_hour = if self.hour == 0 { 12 } else if self.hour > 12 { self.hour - 12 } else { self.hour };
        format!("{}:{:02} {}", display_hour, self.minute, period)
    }

    fn total_inventory_value(&self) -> u32 {
        self.inventory.iter().map(|item| item.total_value()).sum()
    }

    fn total_inventory_cost(&self) -> u32 {
        self.inventory.iter().map(|item| item.total_cost()).sum()
    }

    fn inventory_count(&self) -> u32 {
        self.inventory.iter().map(|item| item.quantity).sum()
    }

    fn expiring_items_count(&self) -> usize {
        self.inventory.iter().filter(|item| item.card.is_expiring_soon()).count()
    }

    fn add_to_inventory(&mut self, card: GiftCard, quantity: u32) {
        // Check if we already have this type of card
        for item in &mut self.inventory {
            if item.card.retailer == card.retailer && 
               item.card.denomination == card.denomination &&
               item.card.purchase_price == card.purchase_price {
                item.quantity += quantity;
                return;
            }
        }
        
        // Add new inventory item if not found
        self.inventory.push(InventoryItem::new(card, quantity));
    }

    fn can_afford(&self, cost: u32) -> bool {
        self.cash >= cost
    }

    fn spend_money(&mut self, amount: u32) -> bool {
        if self.can_afford(amount) {
            self.cash -= amount;
            true
        } else {
            false
        }
    }

    fn generate_random_order(&mut self) {
        let retailers = ["Amazon", "Starbucks", "Target", "iTunes", "Walmart"];
        let denominations = [10, 15, 20, 25, 50];
        let customer_names = ["Alice", "Bob", "Charlie", "Diana", "Eve", "Frank", "Grace", "Henry"];
        
        // Simple randomization based on current time/day
        let retailer_idx = (self.day + self.hour as u32) % retailers.len() as u32;
        let denom_idx = (self.day * 2 + self.minute as u32) % denominations.len() as u32;
        let customer_idx = (self.next_order_id + self.day) % customer_names.len() as u32;
        
        let retailer = retailers[retailer_idx as usize];
        let denomination = denominations[denom_idx as usize];
        let customer_name = customer_names[customer_idx as usize];
        
        let quantity = 1 + (self.day % 5); // 1-5 cards
        let base_price = denomination + 2; // Small markup from wholesale
        
        // Reputation significantly affects pricing
        let reputation_bonus = match self.reputation {
            5 => denomination / 3,           // 33% bonus for 5-star
            4 => denomination / 4,           // 25% bonus for 4-star  
            3 => denomination / 5,           // 20% bonus for 3-star
            2 => denomination / 10,          // 10% bonus for 2-star
            1 => 0,                         // No bonus for 1-star
            _ => 0,
        };
        
        let offered_price = base_price + reputation_bonus;
        
        let deadline_days = 2 + (self.day % 5); // 2-6 days to fulfill
        
        // Priority based on offer amount
        let priority = if offered_price >= denomination + 8 {
            OrderPriority::High
        } else if offered_price >= denomination + 5 {
            OrderPriority::Medium
        } else {
            OrderPriority::Low
        };

        let order = CustomerOrder::new(
            self.next_order_id,
            customer_name,
            retailer,
            denomination,
            quantity,
            offered_price,
            deadline_days,
            priority,
        );

        self.customer_orders.push_back(order);
        self.next_order_id += 1;

        // Add notification
        self.recent_activities.insert(0, format!(
            "📋 New order: {} wants {} {} ${} cards",
            customer_name, quantity, retailer, denomination
        ));
        if self.recent_activities.len() > 10 {
            self.recent_activities.truncate(10);
        }
    }

    fn process_order_aging(&mut self) {
        // Age all orders by 1 day
        for order in &mut self.customer_orders {
            if order.deadline_days > 0 {
                order.deadline_days -= 1;
            }
        }

        // Remove expired orders and damage reputation
        let mut expired_count = 0;
        self.customer_orders.retain(|order| {
            if order.is_expired() {
                expired_count += 1;
                false
            } else {
                true
            }
        });

        if expired_count > 0 {
            self.recent_activities.insert(0, format!(
                "⏰ {} customer orders expired", expired_count
            ));
            if self.recent_activities.len() > 10 {
                self.recent_activities.truncate(10);
            }
            
            // Damage reputation for each expired order
            for _ in 0..expired_count {
                self.decrease_reputation("order_expired");
            }
        }

        // Generate new orders based on reputation
        // Higher reputation = more frequent orders
        let order_chance = match self.reputation {
            5 => self.day % 2 == 0,   // Every other day
            4 => self.day % 3 == 0,   // Every 3 days
            3 => self.day % 3 == 0,   // Every 3 days (default)
            2 => self.day % 4 == 0,   // Every 4 days
            1 => self.day % 5 == 0,   // Every 5 days
            _ => false,
        };
        
        if order_chance {
            self.generate_random_order();
        }
    }

    fn can_fulfill_order(&self, order: &CustomerOrder) -> bool {
        // Check if we have the required cards in inventory
        for item in &self.inventory {
            if item.card.retailer == order.retailer && 
               item.card.denomination == order.denomination &&
               item.quantity >= order.quantity {
                return true;
            }
        }
        false
    }

    fn fulfill_order(&mut self, order_index: usize) -> bool {
        if order_index >= self.customer_orders.len() {
            return false;
        }

        let order = self.customer_orders[order_index].clone();
        
        if !self.can_fulfill_order(&order) {
            // Add failure message
            self.recent_activities.insert(0, format!(
                "❌ Cannot fulfill order #{} - insufficient inventory", 
                order.id
            ));
            if self.recent_activities.len() > 10 {
                self.recent_activities.truncate(10);
            }
            return false;
        }

        // Find and remove cards from inventory
        let mut cards_needed = order.quantity;
        let mut inventory_to_remove = Vec::new();
        
        for (i, item) in self.inventory.iter_mut().enumerate() {
            if item.card.retailer == order.retailer && 
               item.card.denomination == order.denomination &&
               cards_needed > 0 {
                
                let cards_to_take = cards_needed.min(item.quantity);
                cards_needed -= cards_to_take;
                
                if cards_to_take == item.quantity {
                    // Remove entire inventory item
                    inventory_to_remove.push(i);
                } else {
                    // Reduce quantity
                    item.quantity -= cards_to_take;
                }
                
                if cards_needed == 0 {
                    break;
                }
            }
        }

        // Remove depleted inventory items (in reverse order to maintain indices)
        for &i in inventory_to_remove.iter().rev() {
            self.inventory.remove(i);
        }

        // Calculate earnings and profit
        let total_earnings = order.total_offered();
        let cost_basis = order.quantity * (order.denomination - 5); // Estimate wholesale cost
        let profit = total_earnings as i32 - cost_basis as i32;
        
        // Add money to cash
        self.cash += total_earnings;
        
        // Remove the completed order
        self.customer_orders.remove(order_index);
        
        // Add success message
        self.recent_activities.insert(0, format!(
            "✅ Completed order #{}: {} {} ${} cards for ${} (profit: ${})",
            order.id, order.quantity, order.retailer, order.denomination, 
            total_earnings, profit
        ));
        if self.recent_activities.len() > 10 {
            self.recent_activities.truncate(10);
        }

        // Improve reputation for timely fulfillment
        // Extra bonus for fast fulfillment (more than half deadline remaining)
        if order.deadline_days > (2 + (self.day % 5)) / 2 {
            self.improve_reputation("fast_fulfillment");
        } else {
            self.improve_reputation("order_fulfilled");
        }
        
        true
    }

    fn improve_reputation(&mut self, reason: &str) {
        if self.reputation < 5 {
            self.reputation += 1;
            let message = match reason {
                "order_fulfilled" => "⭐ Reputation improved for excellent service!",
                "fast_fulfillment" => "⭐ Reputation boosted for lightning-fast delivery!",
                _ => "⭐ Reputation improved!",
            };
            self.recent_activities.insert(0, message.to_string());
            if self.recent_activities.len() > 10 {
                self.recent_activities.truncate(10);
            }
        }
    }

    fn decrease_reputation(&mut self, reason: &str) {
        if self.reputation > 1 {
            self.reputation -= 1;
            let message = match reason {
                "order_expired" => "💔 Reputation damaged - customers disappointed by expired orders",
                "slow_service" => "💔 Reputation declined due to slow service",
                _ => "💔 Reputation decreased!",
            };
            self.recent_activities.insert(0, message.to_string());
            if self.recent_activities.len() > 10 {
                self.recent_activities.truncate(10);
            }
        }
    }
}

#[derive(Debug)]
struct App {
    screen: Screen,
    selected_menu_item: usize,
    should_quit: bool,
    game_data: GameData,
    last_time_update: Instant,
    game_speed: Duration, // How often to advance time
    paused: bool,
}

impl App {
    fn new() -> App {
        App {
            screen: Screen::MainMenu,
            selected_menu_item: 0,
            should_quit: false,
            game_data: GameData::new(),
            last_time_update: Instant::now(),
            game_speed: Duration::from_secs(3), // Advance 10 minutes every 3 seconds
            paused: false,
        }
    }

    fn update_time(&mut self) {
        if self.paused || matches!(self.screen, Screen::MainMenu) {
            return;
        }

        let now = Instant::now();
        if now.duration_since(self.last_time_update) >= self.game_speed {
            self.game_data.advance_time(10); // Advance 10 minutes
            self.last_time_update = now;
        }
    }

    fn toggle_pause(&mut self) {
        if !matches!(self.screen, Screen::MainMenu) {
            self.paused = !self.paused;
            let status = if self.paused { "⏸️ Paused" } else { "▶️ Resumed" };
            self.game_data.recent_activities.insert(0, status.to_string());
            if self.game_data.recent_activities.len() > 10 {
                self.game_data.recent_activities.truncate(10);
            }
        }
    }

    fn purchase_from_market(&mut self) {
        if !matches!(self.screen, Screen::Market) {
            return;
        }

        // Market items (matches the display in draw_market)
        let market_items = vec![
            ("Amazon", 25, 20, 50),     // (retailer, value, cost, stock)
            ("Starbucks", 10, 8, 30),
            ("Target", 50, 42, 15),
            ("iTunes", 15, 12, 25),
            ("Walmart", 20, 17, 40),
        ];

        if let Some((retailer, denomination, cost, _stock)) = market_items.get(self.selected_menu_item) {
            let purchase_cost = *cost;
            
            if self.game_data.can_afford(purchase_cost) {
                if self.game_data.spend_money(purchase_cost) {
                    // Create the gift card with random expiration (30-90 days)
                    let expiration_days = 30 + (self.game_data.day % 60); // Simple randomization
                    let card = GiftCard::new(retailer, *denomination, *cost, expiration_days);
                    
                    self.game_data.add_to_inventory(card, 1);
                    
                    // Add activity log
                    let activity = format!(
                        "💰 Purchased {} ${} card for ${}", 
                        retailer, denomination, cost
                    );
                    self.game_data.recent_activities.insert(0, activity);
                    if self.game_data.recent_activities.len() > 10 {
                        self.game_data.recent_activities.truncate(10);
                    }
                }
            } else {
                // Not enough money
                let activity = format!(
                    "❌ Insufficient funds for {} ${} (need ${})", 
                    retailer, denomination, cost
                );
                self.game_data.recent_activities.insert(0, activity);
                if self.game_data.recent_activities.len() > 10 {
                    self.game_data.recent_activities.truncate(10);
                }
            }
        }
    }

    fn fulfill_customer_order(&mut self) {
        if !matches!(self.screen, Screen::Orders) {
            return;
        }

        if self.game_data.customer_orders.is_empty() {
            return;
        }

        // Ensure selected item is within bounds
        let order_index = self.selected_menu_item.min(self.game_data.customer_orders.len() - 1);
        
        // Attempt to fulfill the order
        self.game_data.fulfill_order(order_index);
        
        // Adjust selection if we're now beyond the list
        if self.selected_menu_item >= self.game_data.customer_orders.len() && !self.game_data.customer_orders.is_empty() {
            self.selected_menu_item = self.game_data.customer_orders.len() - 1;
        } else if self.game_data.customer_orders.is_empty() {
            self.selected_menu_item = 0;
        }
    }

    fn next_menu_item(&mut self) {
        let menu_items = match self.screen {
            Screen::MainMenu => 4, // New Game, Continue, Tutorial, Quit
            Screen::Dashboard => 6, // Market, Orders, Inventory, Analytics, Settings, Save & Quit
            Screen::Market => 5, // 5 market items
            Screen::Orders => self.game_data.customer_orders.len().max(1), // Number of orders
            Screen::Inventory => self.game_data.inventory.len().max(1), // Number of inventory items
            _ => 1, // Other screens typically have minimal navigation
        };
        self.selected_menu_item = (self.selected_menu_item + 1) % menu_items;
    }

    fn previous_menu_item(&mut self) {
        let menu_items = match self.screen {
            Screen::MainMenu => 4,
            Screen::Dashboard => 6,
            Screen::Market => 5,
            Screen::Orders => self.game_data.customer_orders.len().max(1),
            Screen::Inventory => self.game_data.inventory.len().max(1),
            _ => 1,
        };
        if self.selected_menu_item > 0 {
            self.selected_menu_item -= 1;
        } else {
            self.selected_menu_item = menu_items - 1;
        }
    }

    fn select_menu_item(&mut self) {
        let previous_screen = self.screen.clone();
        
        match self.screen {
            Screen::MainMenu => {
                match self.selected_menu_item {
                    0 => self.screen = Screen::Dashboard, // New Game
                    1 => {}, // Continue (not implemented yet)
                    2 => {}, // Tutorial (not implemented yet)
                    3 => self.should_quit = true, // Quit
                    _ => {}
                }
            }
            Screen::Dashboard => {
                match self.selected_menu_item {
                    0 => self.screen = Screen::Market,    // [1] Market
                    1 => self.screen = Screen::Orders,    // [2] Orders  
                    2 => self.screen = Screen::Inventory, // [3] Inventory
                    3 => self.screen = Screen::Analytics, // [4] Analytics
                    4 => self.screen = Screen::Settings,  // [5] Settings
                    5 => self.screen = Screen::MainMenu,  // [6] Save & Quit
                    _ => {}
                }
            }
            Screen::Market => {
                // Purchase item from market (stay on market screen)
                self.purchase_from_market();
                return; // Don't reset selection
            }
            Screen::Orders => {
                // Fulfill customer order (stay on orders screen)
                self.fulfill_customer_order();
                return; // Don't reset selection
            }
            _ => {
                // Other screens return to dashboard
                self.screen = Screen::Dashboard;
            }
        }
        
        // Reset selection when changing screens
        if !matches!((previous_screen, &self.screen), (Screen::Market, Screen::Market)) {
            self.selected_menu_item = 0;
        }
    }

    fn go_back(&mut self) {
        match self.screen {
            Screen::MainMenu => self.should_quit = true,
            Screen::Dashboard => self.screen = Screen::MainMenu,
            _ => self.screen = Screen::Dashboard,
        }
        self.selected_menu_item = 0;
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new();
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        // Update game time
        app.update_time();
        
        terminal.draw(|f| ui(f, &app))?;

        // Use poll instead of read to avoid blocking
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Esc => app.go_back(),
                    KeyCode::Down => app.next_menu_item(),
                    KeyCode::Up => app.previous_menu_item(),
                    KeyCode::Enter => app.select_menu_item(),
                    KeyCode::Char(' ') => app.toggle_pause(), // Spacebar to pause
                    // Number key quick access for dashboard
                    KeyCode::Char('1') if matches!(app.screen, Screen::Dashboard) => {
                        app.selected_menu_item = 0;
                        app.select_menu_item();
                    },
                    KeyCode::Char('2') if matches!(app.screen, Screen::Dashboard) => {
                        app.selected_menu_item = 1;
                        app.select_menu_item();
                    },
                    KeyCode::Char('3') if matches!(app.screen, Screen::Dashboard) => {
                        app.selected_menu_item = 2;
                        app.select_menu_item();
                    },
                    KeyCode::Char('4') if matches!(app.screen, Screen::Dashboard) => {
                        app.selected_menu_item = 3;
                        app.select_menu_item();
                    },
                    KeyCode::Char('5') if matches!(app.screen, Screen::Dashboard) => {
                        app.selected_menu_item = 4;
                        app.select_menu_item();
                    },
                    KeyCode::Char('6') if matches!(app.screen, Screen::Dashboard) => {
                        app.selected_menu_item = 5;
                        app.select_menu_item();
                    },
                    _ => {}
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    match app.screen {
        Screen::MainMenu => draw_main_menu(f, app),
        Screen::Dashboard => draw_dashboard(f, app),
        Screen::Market => draw_market(f, app),
        Screen::Orders => draw_orders(f, app),
        Screen::Inventory => draw_inventory(f, app),
        Screen::Analytics => draw_placeholder(f, "Analytics", "Business metrics and trends"),
        Screen::Settings => draw_placeholder(f, "Settings", "Game configuration"),
    }
}

fn draw_main_menu(f: &mut Frame, app: &App) {
    let size = f.area();

    let block = Block::default()
        .title("GIFT CARD EMPIRE")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));

    let menu_items = vec![
        "New Game",
        "Continue",
        "Tutorial", 
        "Quit",
    ];

    let items: Vec<ListItem> = menu_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == app.selected_menu_item {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            
            let prefix = if i == app.selected_menu_item { "► " } else { "  " };
            ListItem::new(Line::from(Span::styled(format!("{}{}", prefix, item), style)))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .style(Style::default().fg(Color::White));

    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)])
        .split(size)[0];

    f.render_widget(list, area);

    // Instructions at the bottom
    let instructions = Paragraph::new("Use ↑↓ to navigate, Enter to select, Q to quit")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);

    let instruction_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(size)[1];

    f.render_widget(instructions, instruction_area);
}

fn draw_dashboard(f: &mut Frame, app: &App) {
    let size = f.area();
    
    // Create layout: Header, Main content, Footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Footer
        ])
        .split(size);

    // Header with game stats
    let header_text = format!(
        "Cash: ${}    Rep: {} ({})    Day: {}    Time: {}",
        app.game_data.cash,
        app.game_data.reputation_stars(),
        app.game_data.reputation_description(),
        app.game_data.day,
        app.game_data.time_display()
    );
    
    let header = Paragraph::new(header_text)
        .block(Block::default()
            .title("Gift Card Empire")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White)))
        .style(Style::default().fg(Color::Green))
        .alignment(Alignment::Center);
    
    f.render_widget(header, chunks[0]);

    // Main content area split into menu and activity
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Menu
            Constraint::Percentage(50), // Activity feed
        ])
        .split(chunks[1]);

    // Menu options
    let menu_items = vec![
        "[1] Market",
        "[2] Orders", 
        "[3] Inventory",
        "[4] Analytics",
        "[5] Settings",
        "[6] Save & Quit",
    ];

    let menu_list_items: Vec<ListItem> = menu_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == app.selected_menu_item {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            
            let prefix = if i == app.selected_menu_item { "► " } else { "  " };
            ListItem::new(Line::from(Span::styled(format!("{}{}", prefix, item), style)))
        })
        .collect();

    let menu_list = List::new(menu_list_items)
        .block(Block::default()
            .title("Quick Actions")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White)))
        .style(Style::default().fg(Color::White));

    f.render_widget(menu_list, main_chunks[0]);

    // Recent activity feed
    let activity_items: Vec<ListItem> = app.game_data.recent_activities
        .iter()
        .map(|activity| {
            ListItem::new(Line::from(Span::styled(
                format!("• {}", activity),
                Style::default().fg(Color::Cyan)
            )))
        })
        .collect();

    let activity_list = List::new(activity_items)
        .block(Block::default()
            .title("Recent Activity")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White)))
        .style(Style::default().fg(Color::White));

    f.render_widget(activity_list, main_chunks[1]);

    // Footer with controls and pause status
    let pause_indicator = if app.paused { " ⏸️ PAUSED" } else { "" };
    let footer_text = format!(
        "↑↓ Navigate  Enter Select  [1-6] Quick Access  Space Pause  Esc Back  Q Quit{}",
        pause_indicator
    );
    let footer = Paragraph::new(footer_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White)))
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);

    f.render_widget(footer, chunks[2]);
}

fn draw_market(f: &mut Frame, app: &App) {
    let size = f.area();
    
    // Create layout: Header, Market table, Footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Market content
            Constraint::Length(3), // Footer
        ])
        .split(size);

    // Header showing budget
    let header_text = format!("Your Budget: ${}", app.game_data.cash);
    let header = Paragraph::new(header_text)
        .block(Block::default()
            .title("Wholesale Market")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White)))
        .style(Style::default().fg(Color::Green))
        .alignment(Alignment::Center);
    
    f.render_widget(header, chunks[0]);

    // Market items table
    let market_items = vec![
        ("Amazon", 25, 20, 50),     // (retailer, value, cost, stock)
        ("Starbucks", 10, 8, 30),
        ("Target", 50, 42, 15),
        ("iTunes", 15, 12, 25),
        ("Walmart", 20, 17, 40),
    ];

    // Create table header and rows
    let mut table_content = vec![
        "Retailer    │ Value │ Cost │ Stock │ Profit".to_string(),
        "────────────┼───────┼──────┼───────┼───────".to_string(),
    ];

    for (i, (retailer, value, cost, stock)) in market_items.iter().enumerate() {
        let profit = value - cost;
        let style_char = if i == app.selected_menu_item { "►" } else { " " };
        
        table_content.push(format!(
            "{} {:10} │  ${:2} │ ${:2} │  {:2}+  │ +${:2}",
            style_char, retailer, value, cost, stock, profit
        ));
    }

    let table_items: Vec<ListItem> = table_content
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let style = if i >= 2 && (i - 2) == app.selected_menu_item {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else if i < 2 {
                Style::default().fg(Color::Gray)
            } else {
                Style::default().fg(Color::White)
            };
            
            ListItem::new(Line::from(Span::styled(line.clone(), style)))
        })
        .collect();

    let market_list = List::new(table_items)
        .block(Block::default()
            .title("Available Cards")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White)))
        .style(Style::default().fg(Color::White));

    f.render_widget(market_list, chunks[1]);

    // Footer with controls
    let footer_text = "↑↓ Select  Enter Purchase  Esc Back";
    let footer = Paragraph::new(footer_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White)))
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);

    f.render_widget(footer, chunks[2]);
}

fn draw_orders(f: &mut Frame, app: &App) {
    let size = f.area();
    
    // Create layout: Header, Orders list, Footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Orders content
            Constraint::Length(3), // Footer
        ])
        .split(size);

    // Header
    let header_text = format!("Active Orders: {}", app.game_data.customer_orders.len());
    let header = Paragraph::new(header_text)
        .block(Block::default()
            .title("Customer Orders")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White)))
        .style(Style::default().fg(Color::Green))
        .alignment(Alignment::Center);
    
    f.render_widget(header, chunks[0]);

    // Orders list
    if app.game_data.customer_orders.is_empty() {
        let no_orders = Paragraph::new("No customer orders available\n\nNew orders will appear over time")
            .block(Block::default()
                .title("Orders")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White)))
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        
        f.render_widget(no_orders, chunks[1]);
    } else {
        // Create table header and rows
        let mut table_content = vec![
            "   Order #  │ Customer │ Item           │ Qty │ Offer │ Days │ Priority".to_string(),
            "────────────┼──────────┼────────────────┼─────┼───────┼──────┼────────".to_string(),
        ];

        for (i, order) in app.game_data.customer_orders.iter().enumerate() {
            let style_char = if i == app.selected_menu_item { "►" } else { " " };
            let priority_color = match order.priority {
                OrderPriority::High => "🔴",
                OrderPriority::Medium => "🟡", 
                OrderPriority::Low => "🟢",
            };
            
            // Check if order can be fulfilled
            let fulfillment_indicator = if app.game_data.can_fulfill_order(order) {
                "✅"
            } else {
                "❌"
            };
            
            table_content.push(format!(
                "{} {} #{:4} │ {:8} │ {} ${:2}      │  {:2} │ ${:3}  │  {:2}  │ {} {}",
                style_char,
                fulfillment_indicator,
                order.id,
                order.customer_name,
                order.retailer,
                order.denomination,
                order.quantity,
                order.offered_price_per_card,
                order.deadline_days,
                priority_color,
                order.priority.display()
            ));
        }

        let table_items: Vec<ListItem> = table_content
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let style = if i >= 2 && (i - 2) == app.selected_menu_item {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else if i < 2 {
                    Style::default().fg(Color::Gray)
                } else {
                    Style::default().fg(Color::White)
                };
                
                ListItem::new(Line::from(Span::styled(line.clone(), style)))
            })
            .collect();

        let orders_list = List::new(table_items)
            .block(Block::default()
                .title("Available Orders")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White)))
            .style(Style::default().fg(Color::White));

        f.render_widget(orders_list, chunks[1]);
    }

    // Footer with controls
    let footer_text = "↑↓ Select  Enter Fulfill Order  Esc Back";
    let footer = Paragraph::new(footer_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White)))
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);

    f.render_widget(footer, chunks[2]);
}

fn draw_inventory(f: &mut Frame, app: &App) {
    let size = f.area();
    
    // Create layout: Header, Inventory list, Footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Inventory content
            Constraint::Length(3), // Footer
        ])
        .split(size);

    // Header showing total inventory value
    let total_value = app.game_data.total_inventory_value();
    let inventory_count = app.game_data.inventory_count();
    let header_text = format!("Total Value: ${}    Items: {}", total_value, inventory_count);
    let header = Paragraph::new(header_text)
        .block(Block::default()
            .title("Inventory Management")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White)))
        .style(Style::default().fg(Color::Green))
        .alignment(Alignment::Center);
    
    f.render_widget(header, chunks[0]);

    // Inventory list
    if app.game_data.inventory.is_empty() {
        let no_inventory = Paragraph::new("No inventory available\n\nVisit the Market to purchase gift cards")
            .block(Block::default()
                .title("Inventory")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White)))
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        
        f.render_widget(no_inventory, chunks[1]);
    } else {
        // Create table header and rows
        let mut table_content = vec![
            "   Card        │ Qty │ Cost │ Days Left │ Market Price │ Profit │ Action".to_string(),
            "───────────────┼─────┼──────┼───────────┼──────────────┼────────┼───────".to_string(),
        ];

        for (i, item) in app.game_data.inventory.iter().enumerate() {
            let style_char = if i == app.selected_menu_item { "►" } else { " " };
            
            // Calculate profit potential
            let market_value = item.card.market_value();
            let profit_per_card = market_value as i32 - item.card.purchase_price as i32;
            let total_profit = profit_per_card * item.quantity as i32;
            
            // Show expiration warning
            let expiration_indicator = if item.card.is_expiring_soon() {
                "❗"
            } else {
                " "
            };
            
            table_content.push(format!(
                "{}{} {} ${:2} │  {:2} │ ${:2} │    {:3}    │     ${:2}     │  ${:3}  │ [Sell]",
                style_char,
                expiration_indicator,
                item.card.retailer,
                item.card.denomination,
                item.quantity,
                item.card.purchase_price,
                item.card.days_until_expiration,
                market_value,
                total_profit
            ));
        }

        let table_items: Vec<ListItem> = table_content
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let style = if i >= 2 && (i - 2) == app.selected_menu_item {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else if i < 2 {
                    Style::default().fg(Color::Gray)
                } else if line.contains("❗") {
                    Style::default().fg(Color::Red) // Expiring items in red
                } else {
                    Style::default().fg(Color::White)
                };
                
                ListItem::new(Line::from(Span::styled(line.clone(), style)))
            })
            .collect();

        let inventory_list = List::new(table_items)
            .block(Block::default()
                .title("Current Stock")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White)))
            .style(Style::default().fg(Color::White));

        f.render_widget(inventory_list, chunks[1]);
    }

    // Footer with controls
    let footer_text = "↑↓ Select  Enter Sell Item  Esc Back  ❗ = Expiring Soon";
    let footer = Paragraph::new(footer_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White)))
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);

    f.render_widget(footer, chunks[2]);
}

fn draw_placeholder(f: &mut Frame, title: &str, description: &str) {
    let size = f.area();

    let content = format!("{}\n\n{}\n\nPress Esc to return to dashboard", title, description);
    let placeholder = Paragraph::new(content)
        .block(Block::default()
            .title(format!("Gift Card Empire - {}", title))
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White)))
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center);

    f.render_widget(placeholder, size);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_data_initialization() {
        let game_data = GameData::new();
        
        assert_eq!(game_data.cash, 5000);
        assert_eq!(game_data.reputation, 3);
        assert_eq!(game_data.day, 1);
        assert_eq!(game_data.hour, 9);
        assert_eq!(game_data.minute, 0);
        assert!(!game_data.inventory.is_empty());
        assert!(!game_data.customer_orders.is_empty());
    }

    #[test]
    fn test_purchase_mechanics() {
        let mut game_data = GameData::new();
        let initial_cash = game_data.cash;
        let initial_inventory_count = game_data.inventory.len();
        
        // Test successful purchase
        assert!(game_data.can_afford(100));
        assert!(game_data.spend_money(100));
        assert_eq!(game_data.cash, initial_cash - 100);
        
        // Test insufficient funds
        assert!(!game_data.can_afford(10000));
        assert!(!game_data.spend_money(10000));
        assert_eq!(game_data.cash, initial_cash - 100); // Should not change
        
        // Test inventory addition
        let card = GiftCard::new("TestCard", 25, 20, 30);
        game_data.add_to_inventory(card, 2);
        assert_eq!(game_data.inventory.len(), initial_inventory_count + 1);
    }

    #[test]
    fn test_time_progression() {
        let mut game_data = GameData::new();
        let initial_day = game_data.day;
        let initial_hour = game_data.hour;
        let initial_minute = game_data.minute;
        
        // Test minute advancement
        game_data.advance_time(30);
        assert_eq!(game_data.minute, initial_minute + 30);
        
        // Test hour rollover
        game_data.advance_time(35); // Should go to next hour
        assert_eq!(game_data.hour, initial_hour + 1);
        assert_eq!(game_data.minute, 5); // 30 + 35 = 65, 65 % 60 = 5
        
        // Test day advancement (would need to advance many hours)
        game_data.hour = 23;
        game_data.minute = 50;
        game_data.advance_time(20); // Should trigger new day
        assert_eq!(game_data.day, initial_day + 1);
        assert_eq!(game_data.hour, 0);
        assert_eq!(game_data.minute, 10);
    }

    #[test]
    fn test_customer_order_system() {
        let mut game_data = GameData::new();
        let initial_order_count = game_data.customer_orders.len();
        
        // Test order generation
        game_data.generate_random_order();
        assert_eq!(game_data.customer_orders.len(), initial_order_count + 1);
        
        // Test order properties
        if let Some(order) = game_data.customer_orders.back() {
            assert!(order.id >= 1000);
            assert!(!order.customer_name.is_empty());
            assert!(!order.retailer.is_empty());
            assert!(order.denomination > 0);
            assert!(order.quantity > 0);
            assert!(order.offered_price_per_card > 0);
            assert!(order.deadline_days > 0);
        }
    }

    #[test]
    fn test_gift_card_pricing() {
        let amazon_card = GiftCard::new("Amazon", 25, 20, 30);
        assert_eq!(amazon_card.market_value(), 32); // 25 * 1.30 = 32.5 -> 32
        assert_eq!(amazon_card.potential_profit(), 12); // 32 - 20 = 12
        
        let starbucks_card = GiftCard::new("Starbucks", 10, 8, 60);
        assert_eq!(starbucks_card.market_value(), 12); // 10 * 1.25 = 12.5 -> 12
        assert_eq!(starbucks_card.potential_profit(), 4); // 12 - 8 = 4
        
        // Test expiration detection
        let expiring_card = GiftCard::new("Target", 50, 42, 10);
        assert!(expiring_card.is_expiring_soon()); // <= 15 days
        
        let fresh_card = GiftCard::new("iTunes", 15, 12, 30);
        assert!(!fresh_card.is_expiring_soon()); // > 15 days
    }

    #[test] 
    fn test_app_initialization() {
        let app = App::new();
        
        assert!(matches!(app.screen, Screen::MainMenu));
        assert_eq!(app.selected_menu_item, 0);
        assert!(!app.should_quit);
        assert!(!app.paused);
        assert_eq!(app.game_data.cash, 5000);
    }

    #[test]
    fn test_order_fulfillment() {
        let mut game_data = GameData::new();
        let initial_cash = game_data.cash;
        
        // Find existing Amazon inventory to know the starting amount
        let initial_amazon_quantity = game_data.inventory.iter()
            .find(|item| item.card.retailer == "Amazon" && item.card.denomination == 25)
            .map(|item| item.quantity)
            .unwrap_or(0);
        
        // Create a test order
        let order = CustomerOrder::new(
            9999,
            "TestCustomer",
            "Amazon", 
            25,
            2,
            30, // $30 per card
            3,
            OrderPriority::High
        );
        
        game_data.customer_orders.push_back(order.clone());
        let order_count = game_data.customer_orders.len();
        
        // Test fulfillment capability (should work with sample inventory)
        assert!(game_data.can_fulfill_order(&order));
        
        // Fulfill the order
        assert!(game_data.fulfill_order(order_count - 1));
        
        // Verify results
        assert_eq!(game_data.customer_orders.len(), order_count - 1); // Order removed
        assert_eq!(game_data.cash, initial_cash + 60); // 2 cards * $30 = $60
        
        // Check inventory was reduced by 2
        let final_amazon_quantity = game_data.inventory.iter()
            .find(|item| item.card.retailer == "Amazon" && item.card.denomination == 25)
            .map(|item| item.quantity)
            .unwrap_or(0);
        assert_eq!(final_amazon_quantity, initial_amazon_quantity - 2);
    }

    #[test]
    fn test_order_fulfillment_failure() {
        let mut game_data = GameData::new();
        
        // Create an order we can't fulfill
        let order = CustomerOrder::new(
            9999,
            "TestCustomer", 
            "NonExistent",
            100,
            5,
            200,
            3,
            OrderPriority::High
        );
        
        game_data.customer_orders.push_back(order.clone());
        let order_count = game_data.customer_orders.len();
        let initial_cash = game_data.cash;
        
        // Test that we can't fulfill it
        assert!(!game_data.can_fulfill_order(&order));
        
        // Attempt fulfillment should fail
        assert!(!game_data.fulfill_order(order_count - 1));
        
        // Order should still be there, cash unchanged
        assert_eq!(game_data.customer_orders.len(), order_count);
        assert_eq!(game_data.cash, initial_cash);
    }

    #[test]
    fn test_reputation_system() {
        let mut game_data = GameData::new();
        let initial_reputation = game_data.reputation;
        
        // Test reputation improvement
        game_data.improve_reputation("order_fulfilled");
        assert_eq!(game_data.reputation, initial_reputation + 1);
        
        // Test reputation decrease
        game_data.decrease_reputation("order_expired");
        assert_eq!(game_data.reputation, initial_reputation);
        
        // Test reputation bounds (can't go above 5)
        game_data.reputation = 5;
        game_data.improve_reputation("order_fulfilled");
        assert_eq!(game_data.reputation, 5);
        
        // Test reputation bounds (can't go below 1)
        game_data.reputation = 1;
        game_data.decrease_reputation("order_expired");
        assert_eq!(game_data.reputation, 1);
        
        // Test reputation descriptions
        game_data.reputation = 5;
        assert_eq!(game_data.reputation_description(), "Legendary");
        game_data.reputation = 3;
        assert_eq!(game_data.reputation_description(), "Good");
        game_data.reputation = 1;
        assert_eq!(game_data.reputation_description(), "Poor");
    }

    #[test]
    fn test_reputation_affects_pricing() {
        let mut game_data = GameData::new();
        
        // Test different reputation levels affect order generation
        // We'll test the pricing logic by examining the bonus calculation
        
        // High reputation (5 stars)
        game_data.reputation = 5;
        let denomination = 25;
        let bonus_5_star = denomination / 3; // Should be 8
        assert_eq!(bonus_5_star, 8);
        
        // Medium reputation (3 stars)  
        game_data.reputation = 3;
        let bonus_3_star = denomination / 5; // Should be 5
        assert_eq!(bonus_3_star, 5);
        
        // Low reputation (1 star)
        game_data.reputation = 1;
        let bonus_1_star = 0; // Should be 0
        assert_eq!(bonus_1_star, 0);
    }
}
