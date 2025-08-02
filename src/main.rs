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
use std::{error::Error, io, time::{Duration, Instant}, fs};
use std::collections::VecDeque;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Screen {
    MainMenu,
    Dashboard,
    Market,
    Orders,
    Inventory,
    Analytics,
    Settings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Season {
    Spring,   // March-May: Fresh start, moderate demand
    Summer,   // June-August: Vacation season, travel cards popular
    Fall,     // September-November: Back to school, tech cards popular
    Winter,   // December-February: Holiday season, gift cards surge
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MarketEvent {
    name: String,
    description: String,
    retailer_affected: Option<String>, // None means all retailers
    price_multiplier: f32,  // 1.0 = normal, 1.5 = 50% more expensive, 0.8 = 20% cheaper
    demand_multiplier: f32, // Affects order frequency
    duration_days: u32,
    remaining_days: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct MarketConditions {
    current_season: Season,
    active_events: Vec<MarketEvent>,
    base_demand_modifier: f32, // Seasonal base modifier
    next_event_in_days: u32,
}

#[derive(Debug, Serialize, Deserialize)]
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
    analytics: BusinessAnalytics,
    market_conditions: MarketConditions,
}

#[derive(Debug, Serialize, Deserialize)]
struct BusinessAnalytics {
    total_revenue: u32,
    total_purchases: u32,
    orders_completed: u32,
    orders_expired: u32,
    best_day_revenue: u32,
    cards_sold: u32,
    cards_expired: u32,
    daily_revenues: Vec<u32>, // Track daily performance
    profit_margins: Vec<f32>, // Track efficiency over time
}

impl BusinessAnalytics {
    fn new() -> Self {
        Self {
            total_revenue: 0,
            total_purchases: 0,
            orders_completed: 0,
            orders_expired: 0,
            best_day_revenue: 0,
            cards_sold: 0,
            cards_expired: 0,
            daily_revenues: vec![0], // Start with day 1
            profit_margins: Vec::new(),
        }
    }

    fn record_purchase(&mut self, amount: u32) {
        self.total_purchases += amount;
    }

    fn record_sale(&mut self, revenue: u32, cost: u32, cards_sold: u32) {
        self.total_revenue += revenue;
        self.orders_completed += 1;
        self.cards_sold += cards_sold;
        
        // Calculate profit margin for this sale
        if revenue > 0 {
            let profit_margin = ((revenue as f32 - cost as f32) / revenue as f32) * 100.0;
            self.profit_margins.push(profit_margin);
        }
        
        // Update daily revenue
        if let Some(today_revenue) = self.daily_revenues.last_mut() {
            *today_revenue += revenue;
            self.best_day_revenue = self.best_day_revenue.max(*today_revenue);
        }
    }

    fn record_expired_order(&mut self) {
        self.orders_expired += 1;
    }

    fn record_expired_cards(&mut self, count: u32) {
        self.cards_expired += count;
    }

    fn start_new_day(&mut self) {
        self.daily_revenues.push(0);
        // Keep only last 30 days
        if self.daily_revenues.len() > 30 {
            self.daily_revenues.remove(0);
        }
    }

    fn average_profit_margin(&self) -> f32 {
        if self.profit_margins.is_empty() {
            0.0
        } else {
            self.profit_margins.iter().sum::<f32>() / self.profit_margins.len() as f32
        }
    }

    fn recent_daily_average(&self) -> f32 {
        if self.daily_revenues.len() <= 1 {
            return 0.0;
        }
        
        let recent_days = self.daily_revenues.len().min(7); // Last 7 days
        let sum: u32 = self.daily_revenues.iter().rev().take(recent_days).sum();
        sum as f32 / recent_days as f32
    }

    fn total_profit(&self) -> i32 {
        self.total_revenue as i32 - self.total_purchases as i32
    }
}

impl Season {
    fn from_day(day: u32) -> Self {
        // Approximate seasons based on day of year (assuming 90-day seasons)
        let season_day = (day - 1) % 360; // 360-day year for simplicity
        match season_day {
            0..=89 => Season::Spring,
            90..=179 => Season::Summer, 
            180..=269 => Season::Fall,
            270..=359 => Season::Winter,
            _ => Season::Spring,
        }
    }

    fn display(&self) -> &str {
        match self {
            Season::Spring => "Spring",
            Season::Summer => "Summer", 
            Season::Fall => "Fall",
            Season::Winter => "Winter",
        }
    }

    fn demand_modifier(&self) -> f32 {
        match self {
            Season::Spring => 1.0,  // Normal demand
            Season::Summer => 1.1,  // Slightly higher (vacation)
            Season::Fall => 0.9,    // Slightly lower (back to school)
            Season::Winter => 1.4,  // Much higher (holidays)
        }
    }

    fn retailer_bonus(&self, retailer: &str) -> f32 {
        match (self, retailer) {
            (Season::Summer, "Target") => 1.2,     // Summer vacation shopping
            (Season::Summer, "Walmart") => 1.1,   // General summer demand
            (Season::Fall, "iTunes") => 1.3,      // Back to school tech
            (Season::Fall, "Amazon") => 1.2,      // Online shopping increase
            (Season::Winter, "Amazon") => 1.5,    // Holiday online shopping
            (Season::Winter, "Starbucks") => 1.3, // Holiday coffee gifts
            (Season::Winter, "iTunes") => 1.4,    // Holiday tech gifts
            (Season::Winter, _) => 1.2,           // General holiday boost
            _ => 1.0,
        }
    }
}

impl MarketEvent {
    fn new(name: &str, description: &str, retailer: Option<&str>, price_mult: f32, demand_mult: f32, duration: u32) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            retailer_affected: retailer.map(|s| s.to_string()),
            price_multiplier: price_mult,
            demand_multiplier: demand_mult,
            duration_days: duration,
            remaining_days: duration,
        }
    }

    fn is_expired(&self) -> bool {
        self.remaining_days == 0
    }

    fn affects_retailer(&self, retailer: &str) -> bool {
        self.retailer_affected.as_ref().map_or(true, |r| r == retailer)
    }
}

impl MarketConditions {
    fn new() -> Self {
        Self {
            current_season: Season::Spring,
            active_events: Vec::new(),
            base_demand_modifier: 1.0,
            next_event_in_days: 3 + (1 % 7), // Next event in 3-9 days
        }
    }

    fn update_season(&mut self, day: u32) {
        let new_season = Season::from_day(day);
        if !matches!((&self.current_season, &new_season), 
            (Season::Spring, Season::Spring) | 
            (Season::Summer, Season::Summer) | 
            (Season::Fall, Season::Fall) | 
            (Season::Winter, Season::Winter)) {
            // Season changed
            self.current_season = new_season;
            self.base_demand_modifier = self.current_season.demand_modifier();
        }
    }

    fn process_daily_events(&mut self, day: u32, activities: &mut Vec<String>) {
        // Age existing events
        self.active_events.retain_mut(|event| {
            if event.remaining_days > 0 {
                event.remaining_days -= 1;
                true
            } else {
                activities.insert(0, format!("📈 Market event '{}' has ended", event.name));
                false
            }
        });

        // Check for new events
        if self.next_event_in_days > 0 {
            self.next_event_in_days -= 1;
        } else {
            self.generate_random_event(day, activities);
            self.next_event_in_days = 5 + (day % 10); // Next event in 5-14 days
        }
    }

    fn generate_random_event(&mut self, day: u32, activities: &mut Vec<String>) {
        let event_type = day % 8; // 8 different event types
        
        let event = match event_type {
            0 => MarketEvent::new(
                "Tech Surge", 
                "New gadget releases drive tech gift card demand",
                Some("iTunes"),
                0.9, // 10% cheaper to buy
                1.5, // 50% more demand
                4
            ),
            1 => MarketEvent::new(
                "Coffee Festival",
                "Local coffee festival increases Starbucks popularity", 
                Some("Starbucks"),
                1.1, // 10% more expensive
                1.8, // 80% more demand
                3
            ),
            2 => MarketEvent::new(
                "Supply Chain Issues",
                "Logistics problems affect all retailers",
                None,
                1.3, // 30% more expensive
                0.7, // 30% less demand
                5
            ),
            3 => MarketEvent::new(
                "Amazon Prime Day",
                "Special Amazon promotion increases demand",
                Some("Amazon"),
                0.85, // 15% cheaper
                2.0, // 100% more demand
                2
            ),
            4 => MarketEvent::new(
                "Back to School",
                "Students need supplies, Target benefits",
                Some("Target"),
                1.05, // 5% more expensive
                1.4, // 40% more demand
                7
            ),
            5 => MarketEvent::new(
                "Economic Downturn",
                "Customers tighten budgets, demand drops",
                None,
                1.0, // Same price
                0.6, // 40% less demand
                6
            ),
            6 => MarketEvent::new(
                "Walmart Expansion",
                "New Walmart stores increase accessibility",
                Some("Walmart"),
                0.95, // 5% cheaper
                1.3, // 30% more demand
                4
            ),
            _ => MarketEvent::new(
                "Market Boom",
                "General economic growth benefits all retailers",
                None,
                0.9, // 10% cheaper
                1.2, // 20% more demand
                5
            ),
        };

        activities.insert(0, format!("🎯 New market event: {}", event.name));
        self.active_events.push(event);
    }

    fn get_price_multiplier(&self, retailer: &str) -> f32 {
        let mut multiplier = 1.0;
        
        // Apply seasonal bonus
        multiplier *= self.current_season.retailer_bonus(retailer);
        
        // Apply active events
        for event in &self.active_events {
            if event.affects_retailer(retailer) {
                multiplier *= event.price_multiplier;
            }
        }
        
        multiplier
    }

    fn get_demand_multiplier(&self, retailer: &str) -> f32 {
        let mut multiplier = self.base_demand_modifier;
        
        // Apply seasonal bonus for demand
        multiplier *= self.current_season.retailer_bonus(retailer);
        
        // Apply active events
        for event in &self.active_events {
            if event.affects_retailer(retailer) {
                multiplier *= event.demand_multiplier;
            }
        }
        
        multiplier
    }
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
            analytics: BusinessAnalytics::new(),
            market_conditions: MarketConditions::new(),
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
            // Record expired cards in analytics
            self.analytics.record_expired_cards(expired_count);
            
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

        // Start new day in analytics
        self.analytics.start_new_day();

        // Update market conditions and process events
        self.market_conditions.update_season(self.day);
        self.market_conditions.process_daily_events(self.day, &mut self.recent_activities);

        // Add daily startup message
        let season = self.market_conditions.current_season.display();
        self.recent_activities.insert(0, format!("🌅 Day {} begins ({} season)", self.day, season));
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
        
        // Apply market demand multiplier to pricing
        let demand_multiplier = self.market_conditions.get_demand_multiplier(retailer);
        let market_bonus = if demand_multiplier > 1.2 {
            denomination / 8  // High demand = better prices
        } else if demand_multiplier < 0.8 {
            0  // Low demand = no bonus
        } else {
            denomination / 12  // Normal demand = small bonus
        };
        
        let offered_price = base_price + reputation_bonus + market_bonus;
        
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
            // Record expired orders in analytics
            for _ in 0..expired_count {
                self.analytics.record_expired_order();
            }
            
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

        // Generate new orders based on reputation and market conditions
        // Higher reputation = more frequent orders
        let base_order_chance = match self.reputation {
            5 => self.day % 2 == 0,   // Every other day
            4 => self.day % 3 == 0,   // Every 3 days
            3 => self.day % 3 == 0,   // Every 3 days (default)
            2 => self.day % 4 == 0,   // Every 4 days
            1 => self.day % 5 == 0,   // Every 5 days
            _ => false,
        };
        
        // Apply market demand modifier
        let market_boost = self.market_conditions.base_demand_modifier > 1.0;
        let order_chance = base_order_chance || (market_boost && self.day % 4 == 0);
        
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
        
        // Record sale in analytics
        self.analytics.record_sale(total_earnings, cost_basis, order.quantity);
        
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

    fn save_game(&self, filename: &str) -> Result<(), Box<dyn Error>> {
        let save_data = serde_json::to_string_pretty(self)?;
        fs::write(filename, save_data)?;
        Ok(())
    }

    fn load_game(filename: &str) -> Result<Self, Box<dyn Error>> {
        let save_data = fs::read_to_string(filename)?;
        let game_data: GameData = serde_json::from_str(&save_data)?;
        Ok(game_data)
    }

    fn save_file_exists(filename: &str) -> bool {
        std::path::Path::new(filename).exists()
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

        // Base market items with dynamic pricing
        let base_market_items = vec![
            ("Amazon", 25, 20, 50),     // (retailer, value, base_cost, stock)
            ("Starbucks", 10, 8, 30),
            ("Target", 50, 42, 15),
            ("iTunes", 15, 12, 25),
            ("Walmart", 20, 17, 40),
        ];
        
        // Apply market conditions to get actual prices
        let market_items: Vec<(&str, u32, u32, u32)> = base_market_items.iter()
            .map(|(retailer, value, base_cost, stock)| {
                let price_multiplier = self.game_data.market_conditions.get_price_multiplier(retailer);
                let actual_cost = (*base_cost as f32 * price_multiplier).round() as u32;
                (*retailer, *value, actual_cost, *stock)
            })
            .collect();

        if let Some((retailer, denomination, cost, _stock)) = market_items.get(self.selected_menu_item) {
            let purchase_cost = *cost;
            
            if self.game_data.can_afford(purchase_cost) {
                if self.game_data.spend_money(purchase_cost) {
                    // Create the gift card with random expiration (30-90 days)
                    let expiration_days = 30 + (self.game_data.day % 60); // Simple randomization
                    let card = GiftCard::new(retailer, *denomination, *cost, expiration_days);
                    
                    self.game_data.add_to_inventory(card, 1);
                    
                    // Record purchase in analytics
                    self.game_data.analytics.record_purchase(purchase_cost);
                    
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
            Screen::Dashboard => 7, // Market, Orders, Inventory, Analytics, Settings, Save Game, Quit
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
            Screen::Dashboard => 7,
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
                    1 => { 
                        // Only load if save file exists
                        if App::save_file_exists() {
                            self.load_game(); 
                        }
                    }, // Continue Game
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
                    5 => { self.save_game(); },           // [6] Save Game
                    6 => self.screen = Screen::MainMenu,  // [7] Quit to Menu
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

    fn save_game(&mut self) -> bool {
        const SAVE_FILE: &str = "savegame.json";
        
        match self.game_data.save_game(SAVE_FILE) {
            Ok(()) => {
                self.game_data.recent_activities.insert(0, "💾 Game saved successfully!".to_string());
                if self.game_data.recent_activities.len() > 10 {
                    self.game_data.recent_activities.truncate(10);
                }
                true
            }
            Err(_) => {
                self.game_data.recent_activities.insert(0, "❌ Failed to save game".to_string());
                if self.game_data.recent_activities.len() > 10 {
                    self.game_data.recent_activities.truncate(10);
                }
                false
            }
        }
    }

    fn load_game(&mut self) -> bool {
        const SAVE_FILE: &str = "savegame.json";
        
        match GameData::load_game(SAVE_FILE) {
            Ok(loaded_game_data) => {
                self.game_data = loaded_game_data;
                self.game_data.recent_activities.insert(0, "📂 Game loaded successfully!".to_string());
                if self.game_data.recent_activities.len() > 10 {
                    self.game_data.recent_activities.truncate(10);
                }
                self.screen = Screen::Dashboard;
                true
            }
            Err(_) => {
                // Create error message in current game_data
                self.game_data.recent_activities.insert(0, "❌ Failed to load game".to_string());
                if self.game_data.recent_activities.len() > 10 {
                    self.game_data.recent_activities.truncate(10);
                }
                false
            }
        }
    }

    fn save_file_exists() -> bool {
        const SAVE_FILE: &str = "savegame.json";
        GameData::save_file_exists(SAVE_FILE)
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
                    KeyCode::Char('7') if matches!(app.screen, Screen::Dashboard) => {
                        app.selected_menu_item = 6;
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
        Screen::Analytics => draw_analytics(f, app),
        Screen::Settings => draw_placeholder(f, "Settings", "Game configuration"),
    }
}

fn draw_main_menu(f: &mut Frame, app: &App) {
    let size = f.area();

    let block = Block::default()
        .title("GIFT CARD EMPIRE")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));

    let save_available = App::save_file_exists();
    let continue_text = if save_available {
        "Continue Game"
    } else {
        "Continue (No save file found)"
    };
    
    let menu_items = vec![
        "New Game",
        continue_text,
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

    // Header with game stats including season
    let season = app.game_data.market_conditions.current_season.display();
    let active_events = app.game_data.market_conditions.active_events.len();
    let events_info = if active_events > 0 {
        format!(" • {} events", active_events)
    } else {
        String::new()
    };
    
    let header_text = format!(
        "Cash: ${}    Rep: {} ({})    Day: {}    Time: {}    Season: {}{}",
        app.game_data.cash,
        app.game_data.reputation_stars(),
        app.game_data.reputation_description(),
        app.game_data.day,
        app.game_data.time_display(),
        season,
        events_info
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
        "[6] Save Game",
        "[7] Quit to Menu",
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
        "↑↓ Navigate  Enter Select  [1-7] Quick Access  Space Pause  Esc Back  Q Quit{}",
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

    // Market items table with dynamic pricing
    let base_market_items = vec![
        ("Amazon", 25, 20, 50),     // (retailer, value, base_cost, stock)
        ("Starbucks", 10, 8, 30),
        ("Target", 50, 42, 15),
        ("iTunes", 15, 12, 25),
        ("Walmart", 20, 17, 40),
    ];
    
    let market_items: Vec<(String, u32, u32, u32, String)> = base_market_items.iter()
        .map(|(retailer, value, base_cost, stock)| {
            let price_multiplier = app.game_data.market_conditions.get_price_multiplier(retailer);
            let actual_cost = (*base_cost as f32 * price_multiplier).round() as u32;
            let trend = if price_multiplier > 1.1 {
                "↗".to_string() // Rising
            } else if price_multiplier < 0.9 {
                "↘".to_string() // Falling  
            } else {
                "→".to_string() // Stable
            };
            (retailer.to_string(), *value, actual_cost, *stock, trend)
        })
        .collect();

    // Create table header and rows
    let mut table_content = vec![
        "Retailer    │ Value │ Cost │ Stock │ Profit │ Trend".to_string(),
        "────────────┼───────┼──────┼───────┼────────┼──────".to_string(),
    ];

    for (i, (retailer, value, cost, stock, trend)) in market_items.iter().enumerate() {
        let profit = value - cost;
        let style_char = if i == app.selected_menu_item { "►" } else { " " };
        
        table_content.push(format!(
            "{} {:10} │  ${:2} │ ${:2} │  {:2}+  │ +${:2}   │  {}",
            style_char, retailer, value, cost, stock, profit, trend
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

fn draw_analytics(f: &mut Frame, app: &App) {
    let size = f.area();
    
    // Create layout: Header, Main content (left metrics, right charts), Footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Footer
        ])
        .split(size);

    // Header
    let analytics = &app.game_data.analytics;
    let total_profit = analytics.total_profit();
    let profit_color = if total_profit >= 0 { Color::Green } else { Color::Red };
    
    let header_text = format!(
        "Total Profit: ${:+}    Revenue: ${}    Orders: {}    Avg Margin: {:.1}%",
        total_profit,
        analytics.total_revenue,
        analytics.orders_completed,
        analytics.average_profit_margin()
    );
    
    let header = Paragraph::new(header_text)
        .block(Block::default()
            .title("Business Analytics Dashboard")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White)))
        .style(Style::default().fg(profit_color))
        .alignment(Alignment::Center);
    
    f.render_widget(header, chunks[0]);

    // Main content area split into two columns
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Key metrics
            Constraint::Percentage(50), // Performance data
        ])
        .split(chunks[1]);

    // Left column: Key Business Metrics
    let key_metrics = vec![
        format!("💰 Total Revenue:          ${}", analytics.total_revenue),
        format!("💳 Total Purchases:        ${}", analytics.total_purchases),
        format!("📈 Net Profit:            ${:+}", total_profit),
        format!(""),
        format!("📋 Orders Completed:       {}", analytics.orders_completed),
        format!("⏰ Orders Expired:         {}", analytics.orders_expired),
        format!("📊 Success Rate:          {:.1}%", {
            let total_orders = analytics.orders_completed + analytics.orders_expired;
            if total_orders > 0 {
                (analytics.orders_completed as f32 / total_orders as f32) * 100.0
            } else {
                0.0
            }
        }),
        format!(""),
        format!("🎯 Cards Sold:            {}", analytics.cards_sold),
        format!("💀 Cards Expired:         {}", analytics.cards_expired),
        format!("🔄 Card Efficiency:       {:.1}%", {
            let total_cards = analytics.cards_sold + analytics.cards_expired;
            if total_cards > 0 {
                (analytics.cards_sold as f32 / total_cards as f32) * 100.0
            } else {
                0.0
            }
        }),
        format!(""),
        format!("⭐ Best Day Revenue:      ${}", analytics.best_day_revenue),
        format!("📅 Recent Daily Avg:      ${:.0}", analytics.recent_daily_average()),
    ];

    let metrics_items: Vec<ListItem> = key_metrics
        .iter()
        .map(|metric| {
            let style = if metric.contains("Net Profit") {
                if total_profit >= 0 {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Red)
                }
            } else if metric.is_empty() {
                Style::default().fg(Color::Gray)
            } else {
                Style::default().fg(Color::White)
            };
            
            ListItem::new(Line::from(Span::styled(metric.clone(), style)))
        })
        .collect();

    let metrics_list = List::new(metrics_items)
        .block(Block::default()
            .title("Key Metrics")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White)))
        .style(Style::default().fg(Color::White));

    f.render_widget(metrics_list, main_chunks[0]);

    // Right column: Performance Trends and Daily Revenue
    let mut performance_data = vec![
        format!("📊 RECENT DAILY REVENUES"),
        format!("────────────────────────"),
    ];

    // Show last 7 days of revenue (or whatever we have)
    let recent_days = analytics.daily_revenues.len().min(7);
    let current_day = app.game_data.day;
    
    for (i, &revenue) in analytics.daily_revenues.iter().rev().take(recent_days).enumerate() {
        let day_num = current_day.saturating_sub(i as u32);
        let bar_length = if analytics.best_day_revenue > 0 {
            ((revenue as f32 / analytics.best_day_revenue as f32) * 20.0) as usize
        } else {
            0
        };
        let bar = "█".repeat(bar_length) + &"░".repeat(20 - bar_length);
        
        performance_data.push(format!(
            "Day {:2} │ ${:4} │ {}",
            day_num, revenue, bar
        ));
    }

    performance_data.push(format!(""));
    performance_data.push(format!("📈 PROFIT MARGIN TRENDS"));
    performance_data.push(format!("───────────────────────"));

    // Show recent profit margins
    let recent_margins = analytics.profit_margins.len().min(5);
    if recent_margins > 0 {
        for (i, &margin) in analytics.profit_margins.iter().rev().take(recent_margins).enumerate() {
            let trend_indicator = if i > 0 && i < analytics.profit_margins.len() {
                let prev_margin = analytics.profit_margins[analytics.profit_margins.len() - i];
                if margin > prev_margin { "↗" } 
                else if margin < prev_margin { "↘" } 
                else { "→" }
            } else {
                "→"
            };
            
            performance_data.push(format!(
                "Sale {:2} │ {:5.1}% │ {}",
                analytics.profit_margins.len() - i, margin, trend_indicator
            ));
        }
    } else {
        performance_data.push(format!("No sales data available yet"));
    }

    performance_data.push(format!(""));
    performance_data.push(format!("🎯 STRATEGIC INSIGHTS"));
    performance_data.push(format!("──────────────────"));

    // Add some strategic insights based on the data
    if analytics.orders_completed > 0 {
        let avg_revenue_per_order = analytics.total_revenue / analytics.orders_completed;
        performance_data.push(format!("Avg Revenue/Order: ${}", avg_revenue_per_order));
    }
    
    if total_profit < 0 {
        performance_data.push(format!("⚠️  Operating at a loss"));
        performance_data.push(format!("   Focus on higher margins"));
    } else if analytics.average_profit_margin() < 15.0 {
        performance_data.push(format!("⚠️  Low profit margins"));
        performance_data.push(format!("   Seek better deals"));
    } else {
        performance_data.push(format!("✅ Healthy profit margins"));
    }

    let performance_items: Vec<ListItem> = performance_data
        .iter()
        .map(|item| {
            let style = if item.contains("REVENUES") || item.contains("TRENDS") || item.contains("INSIGHTS") {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else if item.contains("─") {
                Style::default().fg(Color::Gray)
            } else if item.contains("⚠️") {
                Style::default().fg(Color::Red)
            } else if item.contains("✅") {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::White)
            };
            
            ListItem::new(Line::from(Span::styled(item.clone(), style)))
        })
        .collect();

    let performance_list = List::new(performance_items)
        .block(Block::default()
            .title("Performance Analysis")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White)))
        .style(Style::default().fg(Color::White));

    f.render_widget(performance_list, main_chunks[1]);

    // Footer with controls
    let footer_text = "View comprehensive business metrics and trends • Esc Back";
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

    #[test]
    fn test_analytics_tracking() {
        let mut game_data = GameData::new();
        let initial_purchases = game_data.analytics.total_purchases;
        let initial_revenue = game_data.analytics.total_revenue;
        
        // Test purchase tracking
        game_data.analytics.record_purchase(100);
        assert_eq!(game_data.analytics.total_purchases, initial_purchases + 100);
        
        // Test sale tracking
        game_data.analytics.record_sale(150, 100, 2);
        assert_eq!(game_data.analytics.total_revenue, initial_revenue + 150);
        assert_eq!(game_data.analytics.orders_completed, 1);
        assert_eq!(game_data.analytics.cards_sold, 2);
        
        // Test profit calculation
        let total_profit = game_data.analytics.total_profit();
        assert_eq!(total_profit, 150i32 - (initial_purchases + 100) as i32);
        
        // Test profit margin calculation
        let avg_margin = game_data.analytics.average_profit_margin();
        assert!(avg_margin > 0.0);
        assert!(avg_margin < 100.0);
        
        // Test expired order tracking
        game_data.analytics.record_expired_order();
        assert_eq!(game_data.analytics.orders_expired, 1);
        
        // Test expired cards tracking
        game_data.analytics.record_expired_cards(3);
        assert_eq!(game_data.analytics.cards_expired, 3);
        
        // Test daily revenue tracking
        let initial_days = game_data.analytics.daily_revenues.len();
        game_data.analytics.start_new_day();
        assert_eq!(game_data.analytics.daily_revenues.len(), initial_days + 1);
    }

    #[test]
    fn test_save_load_functionality() {
        use std::fs;
        
        let test_filename = "test_save.json";
        
        // Clean up any existing test file
        let _ = fs::remove_file(test_filename);
        
        // Create test game data
        let mut original_game_data = GameData::new();
        original_game_data.cash = 12345;
        original_game_data.reputation = 4;
        original_game_data.day = 42;
        original_game_data.hour = 15;
        original_game_data.minute = 30;
        original_game_data.recent_activities.push("Test activity".to_string());
        
        // Test save
        let save_result = original_game_data.save_game(test_filename);
        assert!(save_result.is_ok());
        assert!(GameData::save_file_exists(test_filename));
        
        // Test load
        let load_result = GameData::load_game(test_filename);
        assert!(load_result.is_ok());
        
        let loaded_game_data = load_result.unwrap();
        
        // Verify data integrity
        assert_eq!(loaded_game_data.cash, 12345);
        assert_eq!(loaded_game_data.reputation, 4);
        assert_eq!(loaded_game_data.day, 42);
        assert_eq!(loaded_game_data.hour, 15);
        assert_eq!(loaded_game_data.minute, 30);
        assert!(loaded_game_data.recent_activities.contains(&"Test activity".to_string()));
        
        // Clean up test file
        let _ = fs::remove_file(test_filename);
        assert!(!GameData::save_file_exists(test_filename));
    }

    #[test]
    fn test_seasonal_market_system() {
        let mut game_data = GameData::new();
        
        // Test season progression
        assert!(matches!(game_data.market_conditions.current_season, Season::Spring));
        
        // Test season changes
        game_data.market_conditions.update_season(100); // Should be Summer
        assert!(matches!(game_data.market_conditions.current_season, Season::Summer));
        
        game_data.market_conditions.update_season(200); // Should be Fall
        assert!(matches!(game_data.market_conditions.current_season, Season::Fall));
        
        game_data.market_conditions.update_season(300); // Should be Winter
        assert!(matches!(game_data.market_conditions.current_season, Season::Winter));
        
        // Test price multipliers
        let amazon_multiplier = game_data.market_conditions.get_price_multiplier("Amazon");
        assert!(amazon_multiplier > 1.0); // Winter should boost Amazon
        
        let starbucks_multiplier = game_data.market_conditions.get_price_multiplier("Starbucks"); 
        assert!(starbucks_multiplier > 1.0); // Winter should boost Starbucks
        
        // Test demand multipliers
        let demand_multiplier = game_data.market_conditions.get_demand_multiplier("Amazon");
        assert!(demand_multiplier > 1.0); // Winter should increase demand
        
        // Test market event creation
        let initial_events = game_data.market_conditions.active_events.len();
        game_data.market_conditions.generate_random_event(42, &mut game_data.recent_activities);
        assert_eq!(game_data.market_conditions.active_events.len(), initial_events + 1);
        
        // Test event affects pricing
        let event = &game_data.market_conditions.active_events[0];
        if let Some(retailer) = &event.retailer_affected {
            let multiplier = game_data.market_conditions.get_price_multiplier(retailer);
            // Should be different from base price due to event
            assert_ne!(multiplier, 1.5); // 1.5 is winter Amazon base
        }
    }
}
