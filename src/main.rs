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
use std::{error::Error, io};

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

        Self {
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
        }
    }

    fn reputation_stars(&self) -> String {
        let filled = "★".repeat(self.reputation as usize);
        let empty = "☆".repeat(5 - self.reputation as usize);
        format!("{}{}", filled, empty)
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
}

#[derive(Debug)]
struct App {
    screen: Screen,
    selected_menu_item: usize,
    should_quit: bool,
    game_data: GameData,
}

impl App {
    fn new() -> App {
        App {
            screen: Screen::MainMenu,
            selected_menu_item: 0,
            should_quit: false,
            game_data: GameData::new(),
        }
    }

    fn next_menu_item(&mut self) {
        let menu_items = match self.screen {
            Screen::MainMenu => 4, // New Game, Continue, Tutorial, Quit
            Screen::Dashboard => 6, // Market, Orders, Inventory, Analytics, Settings, Save & Quit
            Screen::Market => 5, // 5 market items
            _ => 1, // Other screens typically have minimal navigation
        };
        self.selected_menu_item = (self.selected_menu_item + 1) % menu_items;
    }

    fn previous_menu_item(&mut self) {
        let menu_items = match self.screen {
            Screen::MainMenu => 4,
            Screen::Dashboard => 6,
            Screen::Market => 5,
            _ => 1,
        };
        if self.selected_menu_item > 0 {
            self.selected_menu_item -= 1;
        } else {
            self.selected_menu_item = menu_items - 1;
        }
    }

    fn select_menu_item(&mut self) {
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
            _ => {
                // Other screens return to dashboard
                self.screen = Screen::Dashboard;
            }
        }
        self.selected_menu_item = 0; // Reset selection when changing screens
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
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Esc => app.go_back(),
                KeyCode::Down => app.next_menu_item(),
                KeyCode::Up => app.previous_menu_item(),
                KeyCode::Enter => app.select_menu_item(),
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
        Screen::Orders => draw_placeholder(f, "Customer Orders", "Manage customer requests"),
        Screen::Inventory => draw_placeholder(f, "Inventory", "View your gift card stock"),
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
        "Cash: ${}    Rep: {}    Day: {}    Time: {}",
        app.game_data.cash,
        app.game_data.reputation_stars(),
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

    // Footer with controls
    let footer_text = "↑↓ Navigate  Enter Select  [1-6] Quick Access  Esc Back  Q Quit";
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
    let footer_text = "↑↓ Select  Enter Purchase (Coming Soon)  Esc Back";
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
