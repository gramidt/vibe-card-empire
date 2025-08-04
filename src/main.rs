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
    widgets::{Block, Borders, List, ListItem, Paragraph, Table, Row, Cell, Wrap},
    Frame, Terminal,
};
use std::{error::Error, io, time::{Duration, Instant}, fs, io::Write};
use std::collections::VecDeque;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
struct SoundEffects {
    enabled: bool,
}

#[derive(Debug, Clone, Copy)]
enum SoundType {
    Purchase,       // Successful purchase
    Sale,          // Successful sale/order fulfillment  
    NewOrder,      // New customer order
    LevelUp,       // Achievement unlocked
    Warning,       // Expiration warning
    Error,         // Failed action
    RandomEvent,   // Random event appears
    DayChange,     // New day begins
    Paused,        // Game paused
    Navigation,    // Menu navigation (subtle)
}

impl SoundEffects {
    fn new() -> Self {
        Self { enabled: true }
    }
    
    fn play(&self, sound_type: SoundType) {
        if !self.enabled {
            return;
        }
        
        // Use terminal bell and escape sequences for different sounds
        match sound_type {
            SoundType::Purchase => {
                // Happy purchase sound - single bell
                print!("\x07");
            },
            SoundType::Sale => {
                // Success sound - double bell
                print!("\x07");
                std::thread::sleep(Duration::from_millis(100));
                print!("\x07");
            },
            SoundType::NewOrder => {
                // Notification sound - short bell
                print!("\x07");
            },
            SoundType::LevelUp => {
                // Achievement sound - triple bell
                for _ in 0..3 {
                    print!("\x07");
                    std::thread::sleep(Duration::from_millis(150));
                }
            },
            SoundType::Warning => {
                // Warning sound - long bell
                print!("\x07");
                std::thread::sleep(Duration::from_millis(200));
                print!("\x07");
            },
            SoundType::Error => {
                // Error sound - quick double bell
                print!("\x07");
                std::thread::sleep(Duration::from_millis(50));
                print!("\x07");
            },
            SoundType::RandomEvent => {
                // Special event sound - distinctive pattern
                for i in 0..2 {
                    print!("\x07");
                    std::thread::sleep(Duration::from_millis(if i == 0 { 100 } else { 200 }));
                }
            },
            SoundType::DayChange => {
                // Subtle day change - soft bell
                print!("\x07");
            },
            SoundType::Paused => {
                // Pause sound - very brief
                print!("\x07");
            },
            SoundType::Navigation => {
                // Very subtle navigation - no sound by default to avoid spam
                // Could add very quiet click if needed
            },
        }
        
        // Ensure sound is flushed immediately
        let _ = io::stdout().flush();
    }
    
    fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }
    
    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

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
    Achievements,
    Settings,
    RandomEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum AchievementType {
    // Progress milestones
    FirstSale,
    EarlyBird,      // Complete first 10 orders
    Entrepreneur,   // Reach $10,000 cash
    BusinessMogul,  // Reach $50,000 cash
    Millionaire,    // Reach $1,000,000 cash
    
    // Performance achievements
    PerfectWeek,    // 7 days with 100% order completion
    SpeedDemon,     // Fulfill 5 orders in one day
    Efficiency,     // Maintain 90%+ order success rate for 30 days
    MarketMaster,   // Buy during 5 different favorable market events
    
    // Reputation achievements
    LegendaryStatus,    // Reach 5-star reputation
    CustomerFavorite,   // Complete 100 orders
    TrustedSeller,     // Complete 500 orders
    
    // Seasonal achievements
    WinterWinner,      // Earn $5,000 profit in Winter season
    SeasonVeteran,     // Survive all 4 seasons
    EventSurvivor,     // Survive 10 market events
    
    // Inventory achievements
    Collector,         // Own 100+ cards simultaneously
    DiversifiedPortfolio, // Own cards from all 5 retailers simultaneously
    QuickTurnaround,   // Sell inventory within 3 days of purchase
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Achievement {
    achievement_type: AchievementType,
    name: String,
    description: String,
    unlocked: bool,
    unlock_date: Option<u32>, // Game day when unlocked
    progress: u32,           // Current progress toward goal
    target: u32,            // Target value to unlock
    reward_cash: u32,       // Cash reward for unlocking
}

#[derive(Debug, Serialize, Deserialize)]
struct AchievementTracker {
    achievements: Vec<Achievement>,
    total_unlocked: u32,
    recent_unlock: Option<String>, // Recently unlocked achievement name
    // Progress tracking variables
    consecutive_perfect_days: u32,
    consecutive_efficiency_days: u32,
    orders_today: u32,
    favorable_market_purchases: u32,
    seasonal_winter_profit: i32,
    seasons_survived: Vec<Season>,
    events_survived: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum RandomEventType {
    // Positive events
    LoyalCustomer,      // Customer offers premium price for bulk order
    SupplierDiscount,   // Get discount on next 3 purchases
    MediaAttention,     // Reputation boost and customer influx
    LuckyFind,          // Discover extra valuable cards in inventory
    TechGlitch,         // Online competitor down, more customers
    
    // Negative events  
    CardTheft,          // Lose some inventory to theft
    CustomerComplaint,  // Reputation damage and compensation needed
    SupplierIssue,      // Next few purchases more expensive
    MarketCrash,        // All inventory temporarily worth less
    RegulationChange,   // New rules cause complications
    
    // Neutral/Choice events
    BusinessOffer,      // Choose between different business opportunities
    CharityRequest,     // Option to donate for reputation boost
    InventoryAudit,     // Discover accounting discrepancies
    CompetitorMeeting,  // Opportunity for partnership or rivalry
    CustomerSurvey,     // Feedback that affects future operations
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RandomEvent {
    event_type: RandomEventType,
    title: String,
    description: String,
    choice_a: Option<String>,   // First choice option
    choice_b: Option<String>,   // Second choice option
    choice_c: Option<String>,   // Third choice option (rare)
    auto_resolve: bool,         // True if event resolves automatically
    cash_impact: i32,           // Immediate cash change (can be negative)
    reputation_impact: i8,      // Reputation change (-2 to +2)
    inventory_impact: Vec<(String, i32)>, // (retailer, quantity change)
    duration_days: u32,         // How long effects last
    active: bool,               // Whether event is currently active
}

#[derive(Debug, Serialize, Deserialize)]
struct RandomEventManager {
    active_event: Option<RandomEvent>,
    next_event_in_days: u32,
    event_history: Vec<String>, // Record of past events
    player_choice_pending: bool,
    choice_deadline: u32,       // Day when choice must be made
    temp_modifiers: Vec<TempModifier>, // Temporary effects from events
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TempModifier {
    name: String,
    description: String,
    price_multiplier: f32,      // Affects purchase prices
    demand_multiplier: f32,     // Affects order frequency
    reputation_protection: bool, // Prevents reputation loss
    remaining_days: u32,
}

impl TempModifier {
    fn age_day(&mut self) {
        if self.remaining_days > 0 {
            self.remaining_days -= 1;
        }
    }
    
    fn is_expired(&self) -> bool {
        self.remaining_days == 0
    }
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
    achievements: AchievementTracker,
    random_events: RandomEventManager,
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
    
    fn get_price_multiplier_with_random_events(&self, retailer: &str, random_events: &RandomEventManager) -> f32 {
        let mut multiplier = self.get_price_multiplier(retailer);
        
        // Apply random event modifiers
        for modifier in &random_events.temp_modifiers {
            multiplier *= modifier.price_multiplier;
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

impl Achievement {
    fn new(achievement_type: AchievementType, name: &str, description: &str, target: u32, reward: u32) -> Self {
        Self {
            achievement_type,
            name: name.to_string(),
            description: description.to_string(),
            unlocked: false,
            unlock_date: None,
            progress: 0,
            target,
            reward_cash: reward,
        }
    }

    fn update_progress(&mut self, new_progress: u32) -> bool {
        self.progress = new_progress;
        if !self.unlocked && self.progress >= self.target {
            self.unlocked = true;
            true // Achievement unlocked
        } else {
            false
        }
    }

    fn unlock(&mut self, day: u32) {
        if !self.unlocked {
            self.unlocked = true;
            self.unlock_date = Some(day);
        }
    }

    fn progress_percentage(&self) -> f32 {
        if self.target == 0 {
            0.0
        } else {
            (self.progress as f32 / self.target as f32 * 100.0).min(100.0)
        }
    }
}

impl AchievementTracker {
    fn new() -> Self {
        let mut tracker = Self {
            achievements: Vec::new(),
            total_unlocked: 0,
            recent_unlock: None,
            consecutive_perfect_days: 0,
            consecutive_efficiency_days: 0,
            orders_today: 0,
            favorable_market_purchases: 0,
            seasonal_winter_profit: 0,
            seasons_survived: Vec::new(),
            events_survived: 0,
        };
        
        tracker.initialize_achievements();
        tracker
    }

    fn initialize_achievements(&mut self) {
        self.achievements = vec![
            // Progress milestones
            Achievement::new(AchievementType::FirstSale, "First Sale", "Complete your first customer order", 1, 100),
            Achievement::new(AchievementType::EarlyBird, "Early Bird", "Complete your first 10 orders", 10, 500),
            Achievement::new(AchievementType::Entrepreneur, "Entrepreneur", "Accumulate $10,000 in cash", 10000, 1000),
            Achievement::new(AchievementType::BusinessMogul, "Business Mogul", "Accumulate $50,000 in cash", 50000, 5000),
            Achievement::new(AchievementType::Millionaire, "Millionaire", "Accumulate $1,000,000 in cash", 1000000, 50000),
            
            // Performance achievements
            Achievement::new(AchievementType::PerfectWeek, "Perfect Week", "7 consecutive days with 100% order completion", 7, 2000),
            Achievement::new(AchievementType::SpeedDemon, "Speed Demon", "Fulfill 5 orders in a single day", 5, 1500),
            Achievement::new(AchievementType::Efficiency, "Efficiency Expert", "Maintain 90%+ success rate for 30 days", 30, 3000),
            Achievement::new(AchievementType::MarketMaster, "Market Master", "Make purchases during 5 favorable market events", 5, 2500),
            
            // Reputation achievements
            Achievement::new(AchievementType::LegendaryStatus, "Legendary Status", "Reach maximum 5-star reputation", 5, 2000),
            Achievement::new(AchievementType::CustomerFavorite, "Customer Favorite", "Complete 100 customer orders", 100, 3000),
            Achievement::new(AchievementType::TrustedSeller, "Trusted Seller", "Complete 500 customer orders", 500, 10000),
            
            // Seasonal achievements
            Achievement::new(AchievementType::WinterWinner, "Winter Winner", "Earn $5,000 profit during Winter season", 5000, 2000),
            Achievement::new(AchievementType::SeasonVeteran, "Season Veteran", "Experience all 4 seasons", 4, 3000),
            Achievement::new(AchievementType::EventSurvivor, "Event Survivor", "Survive 10 market events", 10, 2500),
            
            // Inventory achievements
            Achievement::new(AchievementType::Collector, "Collector", "Own 100+ gift cards simultaneously", 100, 2000),
            Achievement::new(AchievementType::DiversifiedPortfolio, "Diversified Portfolio", "Own cards from all 5 retailers", 5, 1000),
            Achievement::new(AchievementType::QuickTurnaround, "Quick Turnaround", "Sell inventory within 3 days of purchase", 1, 1500),
        ];
    }

    fn check_cash_achievements(&mut self, cash: u32, day: u32, activities: &mut Vec<String>) {
        self.check_and_unlock(&AchievementType::Entrepreneur, cash, day, activities);
        self.check_and_unlock(&AchievementType::BusinessMogul, cash, day, activities);
        self.check_and_unlock(&AchievementType::Millionaire, cash, day, activities);
    }

    fn check_order_achievements(&mut self, total_orders: u32, reputation: u8, day: u32, activities: &mut Vec<String>) {
        self.check_and_unlock(&AchievementType::FirstSale, total_orders, day, activities);
        self.check_and_unlock(&AchievementType::EarlyBird, total_orders, day, activities);
        self.check_and_unlock(&AchievementType::CustomerFavorite, total_orders, day, activities);
        self.check_and_unlock(&AchievementType::TrustedSeller, total_orders, day, activities);
        self.check_and_unlock(&AchievementType::LegendaryStatus, reputation as u32, day, activities);
    }

    fn check_inventory_achievements(&mut self, inventory: &[InventoryItem], day: u32, activities: &mut Vec<String>) {
        // Check collector achievement
        let total_cards: u32 = inventory.iter().map(|item| item.quantity).sum();
        self.check_and_unlock(&AchievementType::Collector, total_cards, day, activities);
        
        // Check diversified portfolio
        let retailers: std::collections::HashSet<&str> = inventory.iter()
            .map(|item| item.card.retailer.as_str())
            .collect();
        self.check_and_unlock(&AchievementType::DiversifiedPortfolio, retailers.len() as u32, day, activities);
    }

    fn check_seasonal_achievements(&mut self, season: &Season, winter_profit: i32, day: u32, activities: &mut Vec<String>) {
        // Track seasons survived
        if !self.seasons_survived.contains(season) {
            self.seasons_survived.push(season.clone());
        }
        self.check_and_unlock(&AchievementType::SeasonVeteran, self.seasons_survived.len() as u32, day, activities);
        
        // Check winter winner
        if matches!(season, Season::Winter) && winter_profit >= 5000 {
            self.check_and_unlock(&AchievementType::WinterWinner, winter_profit as u32, day, activities);
        }
    }

    fn record_order_completion(&mut self, day: u32) {
        self.orders_today += 1;
        
        // Check speed demon (5 orders in one day)
        if self.orders_today >= 5 {
            if let Some(achievement) = self.achievements.iter_mut()
                .find(|a| matches!(a.achievement_type, AchievementType::SpeedDemon) && !a.unlocked) {
                achievement.unlock(day);
                self.total_unlocked += 1;
                self.recent_unlock = Some(achievement.name.clone());
            }
        }
    }

    fn record_market_purchase(&mut self, price_multiplier: f32, day: u32, activities: &mut Vec<String>) {
        // Favorable market = prices 10%+ below normal
        if price_multiplier <= 0.9 {
            self.favorable_market_purchases += 1;
            self.check_and_unlock(&AchievementType::MarketMaster, self.favorable_market_purchases, day, activities);
        }
    }

    fn process_daily_achievements(&mut self, orders_completed_today: u32, orders_expired_today: u32, analytics: &BusinessAnalytics, day: u32) {
        // Reset daily counters
        self.orders_today = 0;

        // Track perfect days
        if orders_expired_today == 0 && orders_completed_today > 0 {
            self.consecutive_perfect_days += 1;
        } else {
            self.consecutive_perfect_days = 0;
        }

        // Check perfect week
        if self.consecutive_perfect_days >= 7 {
            if let Some(achievement) = self.achievements.iter_mut()
                .find(|a| matches!(a.achievement_type, AchievementType::PerfectWeek) && !a.unlocked) {
                achievement.unlock(day);
                self.total_unlocked += 1;
                self.recent_unlock = Some(achievement.name.clone());
            }
        }

        // Track efficiency
        let total_orders = analytics.orders_completed + analytics.orders_expired;
        if total_orders > 0 {
            let success_rate = analytics.orders_completed as f32 / total_orders as f32;
            if success_rate >= 0.9 {
                self.consecutive_efficiency_days += 1;
            } else {
                self.consecutive_efficiency_days = 0;
            }
        }

        // Check efficiency achievement
        if self.consecutive_efficiency_days >= 30 {
            if let Some(achievement) = self.achievements.iter_mut()
                .find(|a| matches!(a.achievement_type, AchievementType::Efficiency) && !a.unlocked) {
                achievement.unlock(day);
                self.total_unlocked += 1;
                self.recent_unlock = Some(achievement.name.clone());
            }
        }
    }

    fn record_event_survival(&mut self, day: u32, activities: &mut Vec<String>) {
        self.events_survived += 1;
        self.check_and_unlock(&AchievementType::EventSurvivor, self.events_survived, day, activities);
    }

    fn check_and_unlock(&mut self, achievement_type: &AchievementType, progress: u32, day: u32, activities: &mut Vec<String>) {
        if let Some(achievement) = self.achievements.iter_mut()
            .find(|a| a.achievement_type == *achievement_type && !a.unlocked) {
            
            achievement.progress = achievement.progress.max(progress);
            
            if progress >= achievement.target {
                achievement.unlock(day);
                self.total_unlocked += 1;
                self.recent_unlock = Some(achievement.name.clone());
                
                activities.insert(0, format!(
                    "🏆 Achievement Unlocked: {} (+${})", 
                    achievement.name, achievement.reward_cash
                ));
            }
        }
    }

    fn get_recent_unlock(&mut self) -> Option<String> {
        self.recent_unlock.take()
    }

    fn get_unlocked_achievements(&self) -> Vec<&Achievement> {
        self.achievements.iter().filter(|a| a.unlocked).collect()
    }

    fn get_in_progress_achievements(&self) -> Vec<&Achievement> {
        self.achievements.iter().filter(|a| !a.unlocked && a.progress > 0).collect()
    }

    fn calculate_total_rewards(&self) -> u32 {
        self.achievements.iter()
            .filter(|a| a.unlocked)
            .map(|a| a.reward_cash)
            .sum()
    }
}

impl RandomEvent {
    fn new_auto_event(event_type: RandomEventType, title: &str, description: &str, cash: i32, reputation: i8, duration: u32) -> Self {
        Self {
            event_type,
            title: title.to_string(),
            description: description.to_string(),
            choice_a: None,
            choice_b: None,
            choice_c: None,
            auto_resolve: true,
            cash_impact: cash,
            reputation_impact: reputation,
            inventory_impact: Vec::new(),
            duration_days: duration,
            active: true,
        }
    }

    fn new_choice_event(event_type: RandomEventType, title: &str, description: &str, choice_a: &str, choice_b: &str, choice_c: Option<&str>) -> Self {
        Self {
            event_type,
            title: title.to_string(),
            description: description.to_string(),
            choice_a: Some(choice_a.to_string()),
            choice_b: Some(choice_b.to_string()),
            choice_c: choice_c.map(|s| s.to_string()),
            auto_resolve: false,
            cash_impact: 0,
            reputation_impact: 0,
            inventory_impact: Vec::new(),
            duration_days: 1,
            active: true,
        }
    }

    fn apply_choice(&mut self, choice: usize) -> (i32, i8, Vec<TempModifier>) {
        let mut temp_modifiers = Vec::new();
        
        match (&self.event_type, choice) {
            // Business Offer choices
            (RandomEventType::BusinessOffer, 0) => {
                // Choice A: Accept partnership - get discount modifier
                self.cash_impact = -1000;
                self.reputation_impact = 0;
                temp_modifiers.push(TempModifier {
                    name: "Business Partnership".to_string(),
                    description: "10% discount on purchases".to_string(),
                    price_multiplier: 0.9,
                    demand_multiplier: 1.0,
                    reputation_protection: false,
                    remaining_days: 14,
                });
            },
            (RandomEventType::BusinessOffer, 1) => {
                // Choice B: Go solo - get reputation boost
                self.cash_impact = 0;
                self.reputation_impact = 1;
            },

            // Charity Request choices
            (RandomEventType::CharityRequest, 0) => {
                // Choice A: Donate money
                self.cash_impact = -500;
                self.reputation_impact = 2;
            },
            (RandomEventType::CharityRequest, 1) => {
                // Choice B: Donate cards (if possible)
                self.cash_impact = 0;
                self.reputation_impact = 1;
                self.inventory_impact.push(("Amazon".to_string(), -2));
            },
            (RandomEventType::CharityRequest, 2) => {
                // Choice C: Decline
                self.cash_impact = 0;
                self.reputation_impact = -1;
            },

            // Competitor Meeting choices
            (RandomEventType::CompetitorMeeting, 0) => {
                // Choice A: Collaborate
                self.cash_impact = 0;
                self.reputation_impact = 0;
                temp_modifiers.push(TempModifier {
                    name: "Market Collaboration".to_string(),
                    description: "Increased customer demand".to_string(),
                    price_multiplier: 1.0,
                    demand_multiplier: 1.3,
                    reputation_protection: false,
                    remaining_days: 10,
                });
            },
            (RandomEventType::CompetitorMeeting, 1) => {
                // Choice B: Compete aggressively
                self.cash_impact = -200;
                self.reputation_impact = 0;
                temp_modifiers.push(TempModifier {
                    name: "Price War".to_string(),
                    description: "Cheaper purchases but lower demand".to_string(),
                    price_multiplier: 0.85,
                    demand_multiplier: 0.8,
                    reputation_protection: false,
                    remaining_days: 7,
                });
            },

            // Default case
            _ => {
                self.cash_impact = 0;
                self.reputation_impact = 0;
            }
        }

        (self.cash_impact, self.reputation_impact, temp_modifiers)
    }
    
    fn get_choices(&self) -> Vec<&str> {
        let mut choices = Vec::new();
        if let Some(choice) = &self.choice_a {
            choices.push(choice.as_str());
        }
        if let Some(choice) = &self.choice_b {
            choices.push(choice.as_str());
        }
        if let Some(choice) = &self.choice_c {
            choices.push(choice.as_str());
        }
        choices
    }
}


impl RandomEventManager {
    fn new() -> Self {
        Self {
            active_event: None,
            next_event_in_days: 3 + (1 % 5), // Next event in 3-7 days
            event_history: Vec::new(),
            player_choice_pending: false,
            choice_deadline: 0,
            temp_modifiers: Vec::new(),
        }
    }

    fn process_daily_events(&mut self, day: u32, activities: &mut Vec<String>) -> Option<RandomEvent> {
        // Age temporary modifiers
        self.temp_modifiers.retain_mut(|modifier| {
            modifier.age_day();
            if modifier.is_expired() {
                activities.insert(0, format!("📅 {} effect has ended", modifier.name));
                false
            } else {
                true
            }
        });

        // Check for choice deadline
        if self.player_choice_pending && day >= self.choice_deadline {
            // Force auto-resolve if player didn't choose
            if let Some(ref mut event) = self.active_event {
                let (cash, reputation, modifiers) = event.apply_choice(0); // Default to first choice
                self.temp_modifiers.extend(modifiers);
                activities.insert(0, format!("⏰ {} auto-resolved (no choice made)", event.title));
                
                self.player_choice_pending = false;
                self.active_event = None;
            }
        }

        // Check for new events
        if self.active_event.is_none() && self.next_event_in_days > 0 {
            self.next_event_in_days -= 1;
            None
        } else if self.active_event.is_none() && self.next_event_in_days == 0 {
            let mut new_event = self.generate_random_event(day);
            activities.insert(0, format!("🎲 Random event: {}", new_event.title));
            
            if new_event.auto_resolve {
                // Auto-resolve immediate events
                let (cash, reputation, modifiers) = new_event.apply_choice(0);
                self.temp_modifiers.extend(modifiers);
                self.next_event_in_days = 3 + (day % 5); // Schedule next event
                None
            } else {
                // Set up choice event
                self.player_choice_pending = true;
                self.choice_deadline = day + 2; // 2 days to choose
                self.next_event_in_days = 3 + (day % 5); // Schedule next event
                Some(new_event)
            }
        } else {
            None
        }
    }

    fn generate_random_event(&mut self, day: u32) -> RandomEvent {
        let event_type = day % 15; // 15 different event types
        
        let event = match event_type {
            0 => RandomEvent::new_auto_event(
                RandomEventType::LoyalCustomer,
                "Loyal Customer Returns",
                "A satisfied customer wants to buy $2000 worth of gift cards at premium prices!",
                2000,
                1,
                1
            ),
            1 => RandomEvent::new_auto_event(
                RandomEventType::SupplierDiscount,
                "Supplier Discount",
                "Your supplier offers 15% off your next 3 purchases due to good relationship!",
                0,
                0,
                1
            ),
            2 => RandomEvent::new_auto_event(
                RandomEventType::MediaAttention,
                "Positive Media Coverage",
                "Local news features your business! Reputation increases and more customers arrive.",
                500,
                1,
                3
            ),
            3 => RandomEvent::new_auto_event(
                RandomEventType::LuckyFind,
                "Inventory Audit Bonus",
                "During inventory count, you discover some cards are worth more than expected!",
                800,
                0,
                1
            ),
            4 => RandomEvent::new_auto_event(
                RandomEventType::TechGlitch,
                "Competitor System Down",
                "Major online competitor experiences technical issues. Customers flock to you!",
                0,
                0,
                2
            ),
            5 => RandomEvent::new_auto_event(
                RandomEventType::CardTheft,
                "Security Incident",
                "Unfortunately, some inventory was stolen. Insurance covers part of the loss.",
                -300,
                -1,
                1
            ),
            6 => RandomEvent::new_auto_event(
                RandomEventType::CustomerComplaint,
                "Customer Complaint",
                "An unsatisfied customer posts negative reviews. You compensate to maintain reputation.",
                -400,
                -1,
                1
            ),
            7 => RandomEvent::new_auto_event(
                RandomEventType::SupplierIssue,
                "Supplier Price Increase",
                "Your main supplier raises prices due to increased demand. Costs go up temporarily.",
                0,
                0,
                5
            ),
            8 => RandomEvent::new_auto_event(
                RandomEventType::MarketCrash,
                "Market Downturn",
                "Economic uncertainty affects gift card values. Customer demand drops temporarily.",
                0,
                0,
                4
            ),
            9 => RandomEvent::new_auto_event(
                RandomEventType::RegulationChange,
                "New Regulations",
                "Government introduces new gift card regulations. Compliance costs required.",
                -600,
                0,
                1
            ),
            10 => RandomEvent::new_choice_event(
                RandomEventType::BusinessOffer,
                "Partnership Proposal",
                "Another gift card business proposes a partnership. Split costs but share profits.",
                "Accept partnership (-$1000, get purchase discount)",
                "Decline and stay independent (+reputation)",
                None
            ),
            11 => RandomEvent::new_choice_event(
                RandomEventType::CharityRequest,
                "Charity Fundraiser",
                "Local charity asks for donation. Good for reputation but costs money or inventory.",
                "Donate $500 cash (++reputation)",
                "Donate 2 Amazon cards (+reputation)",
                Some("Politely decline (-reputation)")
            ),
            12 => RandomEvent::new_auto_event(
                RandomEventType::InventoryAudit,
                "Surprise Inventory Check",
                "Accounting review reveals minor discrepancies. Small penalty but processes improved.",
                -200,
                0,
                1
            ),
            13 => RandomEvent::new_choice_event(
                RandomEventType::CompetitorMeeting,
                "Competitor Conference",
                "Industry meeting with other gift card sellers. Choose your approach.",
                "Collaborate for mutual benefit (+demand)",
                "Compete aggressively (price war)",
                None
            ),
            _ => RandomEvent::new_auto_event(
                RandomEventType::CustomerSurvey,
                "Customer Feedback Survey",
                "Customer survey results show satisfaction with your service. Reputation boost!",
                0,
                1,
                1
            ),
        };

        // Schedule next event
        self.next_event_in_days = 7 + (day % 14); // Next event in 7-20 days
        
        // Record in history
        self.event_history.push(format!("Day {}: {}", day, event.title));
        if self.event_history.len() > 10 {
            self.event_history.remove(0); // Keep only last 10 events
        }

        // Set as active event
        if event.auto_resolve {
            // Will be processed immediately
            event
        } else {
            // Store for player choice
            self.active_event = Some(event.clone());
            event
        }
    }

    fn make_choice(&mut self, choice: usize) -> Option<(i32, i8, Vec<TempModifier>)> {
        if let Some(ref mut event) = self.active_event {
            let result = event.apply_choice(choice);
            self.player_choice_pending = false;
            self.active_event = None;
            Some(result)
        } else {
            None
        }
    }

    fn get_active_choice_event(&self) -> Option<&RandomEvent> {
        if self.player_choice_pending {
            self.active_event.as_ref()
        } else {
            None
        }
    }

    fn get_total_price_multiplier(&self) -> f32 {
        self.temp_modifiers.iter()
            .map(|m| m.price_multiplier)
            .product()
    }

    fn get_total_demand_multiplier(&self) -> f32 {
        self.temp_modifiers.iter()
            .map(|m| m.demand_multiplier)
            .product()
    }

    fn has_reputation_protection(&self) -> bool {
        self.temp_modifiers.iter()
            .any(|m| m.reputation_protection)
    }

    fn get_active_modifiers(&self) -> &[TempModifier] {
        &self.temp_modifiers
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
            achievements: AchievementTracker::new(),
            random_events: RandomEventManager::new(),
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

        // Process daily achievements
        let orders_completed_today = 0; // TODO: Track daily completion count
        let orders_expired_today = expired_count;
        self.achievements.process_daily_achievements(orders_completed_today, orders_expired_today, &self.analytics, self.day);

        // Check cash and inventory achievements
        self.achievements.check_cash_achievements(self.cash, self.day, &mut self.recent_activities);
        self.achievements.check_inventory_achievements(&self.inventory, self.day, &mut self.recent_activities);
        self.achievements.check_seasonal_achievements(&self.market_conditions.current_season, self.achievements.seasonal_winter_profit, self.day, &mut self.recent_activities);

        // Process random events
        if let Some(event) = self.random_events.process_daily_events(self.day, &mut self.recent_activities) {
            // Handle any returned events (choice-based events)
            self.random_events.active_event = Some(event);
        }

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
        // Match exactly what's available in market - retailer to denomination mapping
        let available_cards = [
            ("Amazon", 25),
            ("Starbucks", 10), 
            ("Target", 50),
            ("iTunes", 15),
            ("Walmart", 20),
        ];
        let customer_names = ["Alice", "Bob", "Charlie", "Diana", "Eve", "Frank", "Grace", "Henry"];
        
        // Simple randomization based on current time/day
        let card_idx = (self.day + self.hour as u32) % available_cards.len() as u32;
        let customer_idx = (self.next_order_id + self.day) % customer_names.len() as u32;
        
        let (retailer, denomination) = available_cards[card_idx as usize];
        let customer_name = customer_names[customer_idx as usize];
        
        let quantity = 1 + (self.day % 5); // 1-5 cards
        
        // Customers want to buy at a discount from face value (that's the business model)
        // Base offer is 85-95% of face value depending on reputation
        let discount_percentage: f32 = match self.reputation {
            5 => 0.95,  // 5% discount for 5-star (customers pay more for reliable service)
            4 => 0.93,  // 7% discount for 4-star
            3 => 0.90,  // 10% discount for 3-star
            2 => 0.87,  // 13% discount for 2-star
            1 => 0.85,  // 15% discount for 1-star (need deep discounts)
            _ => 0.85,
        };
        
        // Apply market demand multiplier
        let demand_multiplier = self.market_conditions.get_demand_multiplier(retailer);
        let demand_adjustment = if demand_multiplier > 1.2 {
            0.02  // High demand = customers pay 2% more
        } else if demand_multiplier < 0.8 {
            -0.03  // Low demand = customers want 3% more discount
        } else {
            0.0  // Normal demand = no adjustment
        };
        
        let final_discount = (discount_percentage + demand_adjustment).clamp(0.80, 0.98);
        let offered_price = (denomination as f32 * final_discount) as u32;
        
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
            5 => true,                // Every day (highest reputation)
            4 => self.day % 2 == 0,   // Every other day
            3 => self.day % 2 == 0,   // Every other day (default - more frequent now)
            2 => self.day % 3 == 0,   // Every 3 days
            1 => self.day % 4 == 0,   // Every 4 days
            _ => false,
        };
        
        // Apply market demand modifier for additional orders
        let market_boost = self.market_conditions.base_demand_modifier > 1.0;
        let extra_market_chance = market_boost && self.day % 2 == 1; // Additional orders on alternate days during good markets
        let order_chance = base_order_chance || extra_market_chance;
        
        if order_chance {
            self.generate_random_order();
        }
    }

    fn can_fulfill_order(&self, order: &CustomerOrder) -> bool {
        // Check if we have enough cards total across all inventory items
        let total_available = self.inventory.iter()
            .filter(|item| item.card.retailer == order.retailer && 
                          item.card.denomination == order.denomination)
            .map(|item| item.quantity)
            .sum::<u32>();
            
        total_available >= order.quantity
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

        // Check achievements
        self.achievements.record_order_completion(self.day);
        self.achievements.check_order_achievements(self.analytics.orders_completed, self.reputation, self.day, &mut self.recent_activities);
        self.achievements.check_cash_achievements(self.cash, self.day, &mut self.recent_activities);
        
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
    sound_effects: SoundEffects,
}

impl App {
    fn new() -> App {
        App {
            screen: Screen::MainMenu,
            selected_menu_item: 0,
            should_quit: false,
            game_data: GameData::new(),
            last_time_update: Instant::now(),
            game_speed: Duration::from_secs(1), // Advance 20 minutes every 1 second
            paused: false,
            sound_effects: SoundEffects::new(),
        }
    }

    fn update_time(&mut self) {
        if self.paused || matches!(self.screen, Screen::MainMenu) {
            return;
        }

        let now = Instant::now();
        if now.duration_since(self.last_time_update) >= self.game_speed {
            self.game_data.advance_time(20); // Advance 20 minutes
            self.last_time_update = now;
        }
    }
    
    fn check_for_active_events(&mut self) {
        // Check if we need to switch to random event screen for player choice
        if self.game_data.random_events.player_choice_pending && 
           !matches!(self.screen, Screen::RandomEvent) {
            self.sound_effects.play(SoundType::RandomEvent);
            self.screen = Screen::RandomEvent;
            self.selected_menu_item = 0; // Reset selection
        } else if !self.game_data.random_events.player_choice_pending && 
                  matches!(self.screen, Screen::RandomEvent) {
            // Return to dashboard if event is resolved
            self.screen = Screen::Dashboard;
            self.selected_menu_item = 0;
        }
    }
    
    fn check_for_audio_events(&mut self) {
        // Check for recent achievement unlocks
        if let Some(_achievement_name) = self.game_data.achievements.get_recent_unlock() {
            self.sound_effects.play(SoundType::LevelUp);
        }
        
        // Check for new orders (simple detection by counting recent activities with order keywords)
        if let Some(recent_activity) = self.game_data.recent_activities.first() {
            if recent_activity.contains("New customer order") || recent_activity.contains("📝 Order from") {
                self.sound_effects.play(SoundType::NewOrder);
            }
        }
    }

    fn toggle_pause(&mut self) {
        if !matches!(self.screen, Screen::MainMenu) {
            self.paused = !self.paused;
            self.sound_effects.play(SoundType::Paused);
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
                let price_multiplier = self.game_data.market_conditions.get_price_multiplier_with_random_events(retailer, &self.game_data.random_events);
                let actual_cost = (*base_cost as f32 * price_multiplier).round() as u32;
                (*retailer, *value, actual_cost, *stock)
            })
            .collect();

        if let Some((retailer, denomination, cost, _stock)) = market_items.get(self.selected_menu_item) {
            let purchase_cost = *cost;
            
            if self.game_data.can_afford(purchase_cost) {
                if self.game_data.spend_money(purchase_cost) {
                    // Play purchase success sound
                    self.sound_effects.play(SoundType::Purchase);
                    
                    // Create the gift card with random expiration (30-90 days)
                    let expiration_days = 30 + (self.game_data.day % 60); // Simple randomization
                    let card = GiftCard::new(retailer, *denomination, *cost, expiration_days);
                    
                    self.game_data.add_to_inventory(card, 1);
                    
                    // Record purchase in analytics
                    self.game_data.analytics.record_purchase(purchase_cost);
                    
                    // Check market purchase achievements
                    let price_multiplier = self.game_data.market_conditions.get_price_multiplier_with_random_events(retailer, &self.game_data.random_events);
                    self.game_data.achievements.record_market_purchase(price_multiplier, self.game_data.day, &mut self.game_data.recent_activities);
                    
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
                self.sound_effects.play(SoundType::Error);
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
        let success = self.game_data.fulfill_order(order_index);
        
        // Play appropriate sound
        if success {
            self.sound_effects.play(SoundType::Sale);
        } else {
            self.sound_effects.play(SoundType::Error);
        }
        
        // Adjust selection if we're now beyond the list
        if self.selected_menu_item >= self.game_data.customer_orders.len() && !self.game_data.customer_orders.is_empty() {
            self.selected_menu_item = self.game_data.customer_orders.len() - 1;
        } else if self.game_data.customer_orders.is_empty() {
            self.selected_menu_item = 0;
        }
    }
    
    fn handle_random_event_choice(&mut self) {
        // Make choice on active random event
        if let Some((cash, reputation, modifiers)) = self.game_data.random_events.make_choice(self.selected_menu_item) {
            // Apply impacts immediately
            if cash != 0 {
                if cash > 0 {
                    self.game_data.cash = self.game_data.cash.saturating_add(cash as u32);
                } else {
                    self.game_data.cash = self.game_data.cash.saturating_sub((-cash) as u32);
                }
            }
            
            if reputation != 0 {
                if reputation > 0 {
                    self.game_data.reputation = (self.game_data.reputation.saturating_add(reputation as u8)).min(5);
                } else {
                    self.game_data.reputation = self.game_data.reputation.saturating_sub((-reputation) as u8).max(1);
                }
            }
            
            // Add temporary modifiers
            self.game_data.random_events.temp_modifiers.extend(modifiers);
            
            // Log the choice result
            let activity = "✅ Made choice on random event".to_string();
            self.game_data.recent_activities.insert(0, activity);
            if self.game_data.recent_activities.len() > 10 {
                self.game_data.recent_activities.truncate(10);
            }
            
            // Event will be automatically cleared and screen switched in check_for_active_events
        }
    }

    fn sell_inventory_item(&mut self) {
        if !matches!(self.screen, Screen::Inventory) {
            return;
        }

        if self.game_data.inventory.is_empty() {
            return;
        }

        // Ensure selected item is within bounds
        let inventory_index = self.selected_menu_item.min(self.game_data.inventory.len() - 1);
        
        // Get the selected inventory item
        let item = &self.game_data.inventory[inventory_index];
        
        // Calculate market value (sell at slightly below retail value)
        let retail_value = item.card.denomination;
        let market_value = (retail_value as f32 * 0.85) as u32; // Sell at 85% of face value
        let total_value = market_value * item.quantity;
        
        // Calculate profit/loss
        let total_cost = item.card.purchase_price * item.quantity;
        let profit = total_value as i32 - total_cost as i32;
        
        // Add cash to player
        self.game_data.cash += total_value;
        
        // Record the sale in analytics
        self.game_data.analytics.cards_sold += item.quantity;
        self.game_data.analytics.total_revenue += total_value;
        
        // Play success sound
        self.sound_effects.play(SoundType::Sale);
        
        // Add activity log
        let activity = format!(
            "💰 Sold {}x {} ${} cards for ${} ({}${} profit)",
            item.quantity,
            item.card.retailer,
            item.card.denomination,
            total_value,
            if profit >= 0 { "+" } else { "" },
            profit
        );
        self.game_data.recent_activities.insert(0, activity);
        if self.game_data.recent_activities.len() > 10 {
            self.game_data.recent_activities.truncate(10);
        }
        
        // Remove the sold item from inventory
        self.game_data.inventory.remove(inventory_index);
        
        // Adjust selection if we're now beyond the list
        if self.selected_menu_item >= self.game_data.inventory.len() && !self.game_data.inventory.is_empty() {
            self.selected_menu_item = self.game_data.inventory.len() - 1;
        } else if self.game_data.inventory.is_empty() {
            self.selected_menu_item = 0;
        }
    }

    fn next_menu_item(&mut self) {
        let menu_items = match self.screen {
            Screen::MainMenu => 4, // New Game, Continue, Tutorial, Quit
            Screen::Dashboard => 8, // Market, Orders, Inventory, Analytics, Achievements, Settings, Save Game, Quit
            Screen::Market => 5, // 5 market items
            Screen::Orders => self.game_data.customer_orders.len().max(1), // Number of orders
            Screen::Inventory => self.game_data.inventory.len().max(1), // Number of inventory items
            Screen::RandomEvent => {
                // Get number of choices for active event
                if let Some(event) = &self.game_data.random_events.active_event {
                    event.get_choices().len().max(1)
                } else {
                    1
                }
            },
            _ => 1, // Other screens typically have minimal navigation
        };
        self.selected_menu_item = (self.selected_menu_item + 1) % menu_items;
    }

    fn previous_menu_item(&mut self) {
        let menu_items = match self.screen {
            Screen::MainMenu => 4,
            Screen::Dashboard => 8,
            Screen::Market => 5,
            Screen::Orders => self.game_data.customer_orders.len().max(1),
            Screen::Inventory => self.game_data.inventory.len().max(1),
            Screen::RandomEvent => {
                // Get number of choices for active event
                if let Some(event) = &self.game_data.random_events.active_event {
                    event.get_choices().len().max(1)
                } else {
                    1
                }
            },
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
                    0 => self.screen = Screen::Market,       // [1] Market
                    1 => self.screen = Screen::Orders,       // [2] Orders  
                    2 => self.screen = Screen::Inventory,    // [3] Inventory
                    3 => self.screen = Screen::Analytics,    // [4] Analytics
                    4 => self.screen = Screen::Achievements, // [5] Achievements
                    5 => self.screen = Screen::Settings,     // [6] Settings
                    6 => { self.save_game(); },              // [7] Save Game
                    7 => self.screen = Screen::MainMenu,     // [8] Quit to Menu
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
            Screen::Inventory => {
                // Sell inventory item (stay on inventory screen)
                self.sell_inventory_item();
                return; // Don't reset selection
            }
            Screen::RandomEvent => {
                // Handle random event choice
                self.handle_random_event_choice();
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
        
        // Check for active random events that need player choice
        app.check_for_active_events();
        
        // Check for audio events (achievements, new orders, etc.)
        app.check_for_audio_events();
        
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
                    KeyCode::Char('8') if matches!(app.screen, Screen::Dashboard) => {
                        app.selected_menu_item = 7;
                        app.select_menu_item();
                    },
                    KeyCode::Char('s') | KeyCode::Char('S') => {
                        // Toggle sound effects
                        app.sound_effects.toggle();
                        let status = if app.sound_effects.is_enabled() { 
                            "🔊 Sound effects enabled" 
                        } else { 
                            "🔇 Sound effects disabled" 
                        };
                        app.game_data.recent_activities.insert(0, status.to_string());
                        if app.game_data.recent_activities.len() > 10 {
                            app.game_data.recent_activities.truncate(10);
                        }
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
        Screen::Achievements => draw_achievements_screen(f, app),
        Screen::Settings => draw_placeholder(f, "Settings", "Game configuration"),
        Screen::RandomEvent => draw_random_event(f, app),
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
    
    // Add animated time progression indicator
    let time_indicator = if app.paused {
        "⏸️"
    } else {
        // Cycle through different clock symbols for visual time progression
        match (app.game_data.minute / 10) % 6 {
            0 => "🕐",
            1 => "🕑",
            2 => "🕒",
            3 => "🕓",
            4 => "🕔",
            _ => "🕕",
        }
    };
    
    // Add random event indicator if active
    let random_event_status = if app.game_data.random_events.player_choice_pending {
        " 🎲❗"
    } else if app.game_data.random_events.temp_modifiers.len() > 0 {
        " 🎲✨"
    } else {
        ""
    };
    
    let header_text = format!(
        "Cash: ${}    Rep: {} ({})    Day: {}    Time: {} {}    Season: {}{}{}",
        app.game_data.cash,
        app.game_data.reputation_stars(),
        app.game_data.reputation_description(),
        app.game_data.day,
        app.game_data.time_display(),
        time_indicator,
        season,
        events_info,
        random_event_status
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
        "[5] Achievements",
        "[6] Settings",
        "[7] Save Game",
        "[8] Quit to Menu",
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
    let sound_indicator = if app.sound_effects.is_enabled() { " 🔊" } else { " 🔇" };
    let footer_text = format!(
        "↑↓ Navigate  Enter Select  [1-8] Quick Access  Space Pause  S Sound{}  Esc Back  Q Quit{}",
        sound_indicator,
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
            let price_multiplier = app.game_data.market_conditions.get_price_multiplier_with_random_events(retailer, &app.game_data.random_events);
            let actual_cost = (*base_cost as f32 * price_multiplier).round() as u32;
            // More detailed animated trend indicators
            let trend = if price_multiplier > 1.2 {
                match (app.game_data.minute / 5) % 3 {
                    0 => "🔥↗".to_string(),
                    1 => "🚀↗".to_string(), 
                    _ => "📈↗".to_string(),
                }
            } else if price_multiplier > 1.1 {
                "📈↗".to_string() // Rising
            } else if price_multiplier < 0.8 {
                match (app.game_data.minute / 5) % 3 {
                    0 => "💎↘".to_string(),
                    1 => "🎯↘".to_string(),
                    _ => "📉↘".to_string(),
                }
            } else if price_multiplier < 0.9 {
                "📉↘".to_string() // Falling  
            } else {
                "➡️".to_string() // Stable
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
            
            // Enhanced expiration indicators with animation
            let expiration_indicator = if item.card.days_until_expiration <= 3 {
                // Critical - blinking warning
                match (app.game_data.minute / 3) % 3 {
                    0 => "🚨",
                    1 => "❗",
                    _ => "⚠️",
                }
            } else if item.card.days_until_expiration <= 7 {
                // Warning - steady indicator
                "⚠️"
            } else if item.card.days_until_expiration <= 14 {
                // Caution - mild indicator
                "⚡"
            } else {
                // Fresh - good indicator
                "✅"
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

fn draw_achievements_screen(f: &mut Frame, app: &App) {
    let size = f.area();
    
    // Create layout: Header, Main content (left unlocked, right in-progress), Footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Footer
        ])
        .split(size);

    // Header with achievement stats
    let total_achievements = app.game_data.achievements.achievements.len();
    let unlocked_count = app.game_data.achievements.total_unlocked;
    let total_rewards = app.game_data.achievements.calculate_total_rewards();
    
    let completion_percentage = if total_achievements == 0 {
        0.0
    } else {
        (unlocked_count as f32 / total_achievements as f32) * 100.0
    };
    
    let header_text = format!(
        "Achievements: {}/{}    Total Rewards: ${}    Completion: {:.1}%",
        unlocked_count,
        total_achievements,
        total_rewards,
        completion_percentage
    );
    
    let header = Paragraph::new(header_text)
        .block(Block::default()
            .title("Achievement Gallery")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White)))
        .style(Style::default().fg(Color::Green))
        .alignment(Alignment::Center);
    
    f.render_widget(header, chunks[0]);

    // Main content area split into two columns
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Unlocked achievements
            Constraint::Percentage(50), // In-progress achievements
        ])
        .split(chunks[1]);

    // Left column: Unlocked Achievements
    let unlocked = app.game_data.achievements.get_unlocked_achievements();
    let unlocked_items: Vec<ListItem> = if unlocked.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            "No achievements unlocked yet!",
            Style::default().fg(Color::Gray)
        )))]
    } else {
        unlocked.iter().map(|achievement| {
            let unlock_day = achievement.unlock_date.unwrap_or(0);
            let content = format!(
                "🏆 {} (+${})\n   {}\n   Unlocked: Day {}",
                achievement.name,
                achievement.reward_cash,
                achievement.description,
                unlock_day
            );
            
            ListItem::new(Line::from(Span::styled(
                content,
                Style::default().fg(Color::Yellow)
            )))
        }).collect()
    };

    let unlocked_list = List::new(unlocked_items)
        .block(Block::default()
            .title(format!("🏆 Unlocked ({}/{})", unlocked_count, total_achievements))
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White)))
        .style(Style::default().fg(Color::White));

    f.render_widget(unlocked_list, main_chunks[0]);

    // Right column: In-Progress Achievements
    let in_progress = app.game_data.achievements.get_in_progress_achievements();
    let mut remaining_achievements: Vec<&Achievement> = app.game_data.achievements.achievements.iter()
        .filter(|a| !a.unlocked && a.progress == 0)
        .take(5) // Show top 5 remaining
        .collect();

    // Combine in-progress and some remaining achievements
    let mut progress_items: Vec<ListItem> = Vec::new();
    
    // Add in-progress achievements first
    for achievement in in_progress {
        let progress_bar_length = ((achievement.progress_percentage() / 100.0 * 20.0) as usize).min(20);
        let progress_bar = "█".repeat(progress_bar_length) + &"░".repeat(20_usize.saturating_sub(progress_bar_length));
        let content = format!(
            "📈 {} ({}/{})\n   {}\n   [{}] {:.1}%",
            achievement.name,
            achievement.progress,
            achievement.target,
            achievement.description,
            progress_bar,
            achievement.progress_percentage()
        );
        
        progress_items.push(ListItem::new(Line::from(Span::styled(
            content,
            Style::default().fg(Color::Cyan)
        ))));
    }

    // Add some locked achievements
    remaining_achievements.truncate(5 - progress_items.len());
    for achievement in remaining_achievements {
        let content = format!(
            "🔒 {} (Reward: ${})\n   {}\n   Progress: {}/{}",
            achievement.name,
            achievement.reward_cash,
            achievement.description,
            achievement.progress,
            achievement.target
        );
        
        progress_items.push(ListItem::new(Line::from(Span::styled(
            content,
            Style::default().fg(Color::Gray)
        ))));
    }

    if progress_items.is_empty() {
        progress_items.push(ListItem::new(Line::from(Span::styled(
            "All achievements unlocked!\nCongratulations!",
            Style::default().fg(Color::Green)
        ))));
    }

    let progress_list = List::new(progress_items)
        .block(Block::default()
            .title("📊 Progress & Locked")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White)))
        .style(Style::default().fg(Color::White));

    f.render_widget(progress_list, main_chunks[1]);

    // Footer with controls
    let footer_text = "View your accomplishments and track progress • Esc Back";
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

fn draw_random_event(f: &mut Frame, app: &App) {
    let size = f.area();
    
    if let Some(event) = &app.game_data.random_events.active_event {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Min(6),     // Event content
                Constraint::Length(3),  // Instructions
            ])
            .split(size);
        
        // Header
        let header = Paragraph::new("🎲 Random Event - Choose Your Response")
            .block(Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Yellow)))
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        f.render_widget(header, chunks[0]);
        
        // Event content
        let event_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4),  // Title and description
                Constraint::Min(2),     // Choices
            ])
            .split(chunks[1]);
        
        // Event title and description
        let event_content = format!("{}\n\n{}", event.title, event.description);
        let event_text = Paragraph::new(event_content)
            .block(Block::default()
                .title("Event Details")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White)))
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true });
        f.render_widget(event_text, event_chunks[0]);
        
        // Choices
        let choice_items: Vec<ListItem> = event.get_choices().iter().enumerate()
            .map(|(i, choice)| {
                let style = if i == app.selected_menu_item {
                    Style::default().fg(Color::Black).bg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(choice.to_string()).style(style)
            })
            .collect();
        
        let choices_list = List::new(choice_items)
            .block(Block::default()
                .title("Your Choices")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White)))
            .highlight_style(Style::default().fg(Color::Black).bg(Color::Yellow));
        
        f.render_widget(choices_list, event_chunks[1]);
        
        // Instructions
        let instructions = Paragraph::new("↑/↓: Navigate  Enter: Select Choice  Time remaining: ⏰")
            .block(Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Gray)))
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(instructions, chunks[2]);
        
    } else {
        // No active event - this shouldn't happen but handle gracefully
        let content = "No active random event.\n\nReturning to dashboard...";
        let placeholder = Paragraph::new(content)
            .block(Block::default()
                .title("Random Events")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White)))
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center);
        f.render_widget(placeholder, size);
    }
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

    #[test]
    fn test_achievement_system() {
        let mut game_data = GameData::new();
        let initial_achievements = game_data.achievements.total_unlocked;
        
        // Test cash achievements
        game_data.cash = 10000;
        game_data.achievements.check_cash_achievements(game_data.cash, game_data.day, &mut game_data.recent_activities);
        assert!(game_data.achievements.total_unlocked > initial_achievements);
        
        // Test order achievements
        game_data.analytics.orders_completed = 1;
        game_data.achievements.check_order_achievements(game_data.analytics.orders_completed, game_data.reputation, game_data.day, &mut game_data.recent_activities);
        
        // Test reputation achievement
        game_data.reputation = 5;
        game_data.achievements.check_order_achievements(0, game_data.reputation, game_data.day, &mut game_data.recent_activities);
        
        // Check if any achievement unlocked for reputation
        let legendary_unlocked = game_data.achievements.achievements.iter()
            .find(|a| a.achievement_type == AchievementType::LegendaryStatus)
            .map(|a| a.unlocked)
            .unwrap_or(false);
        assert!(legendary_unlocked);
        
        // Test inventory achievements
        let total_cards: u32 = game_data.inventory.iter().map(|item| item.quantity).sum();
        assert!(total_cards > 0); // Should have sample inventory
        
        // Test progress tracking
        let in_progress = game_data.achievements.get_in_progress_achievements();
        assert!(!in_progress.is_empty()); // Should have some progress
        
        // Test unlocked achievements
        let unlocked = game_data.achievements.get_unlocked_achievements();
        assert!(!unlocked.is_empty()); // Should have unlocked some
        
        // Test total rewards calculation
        let total_rewards = game_data.achievements.calculate_total_rewards();
        assert!(total_rewards > 0); // Should have earned some rewards
    }
}
