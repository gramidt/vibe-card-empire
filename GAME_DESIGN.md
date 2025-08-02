# Gift Card Empire - Game Design Document

## Overview

**Title:** Gift Card Empire  
**Genre:** Business Simulation / Strategy  
**Platform:** Terminal User Interface (TUI)  
**Target Audience:** Strategy game enthusiasts, business simulation fans  

### Core Concept
A business simulation game where players manage a gift card distribution company, buying wholesale gift cards and selling them to customers for profit while managing inventory, reputation, and market dynamics.

## Game Mechanics

### Core Loop
1. **Purchase** - Buy gift cards at wholesale prices from various retailers
2. **Manage** - Monitor inventory, track expiration dates, optimize storage
3. **Fulfill** - Accept and complete customer orders for specific gift cards
4. **Profit** - Earn money through strategic pricing and efficient operations

### Key Systems

#### Economy System
- **Starting Capital**: $2,500 (Hard) / $5,000 (Normal) / $10,000 (Easy)
- **Revenue Sources**: Customer orders, bulk sales
- **Expenses**: Wholesale purchases, expired inventory losses
- **Profit Margins**: 15-30% typical markup depending on card type and market conditions

#### Inventory Management
- **Expiration Dates**: All gift cards have expiration timers (15-120 days)
- **Storage Limits**: Physical storage constraints require strategic planning
- **Card Types**: Different retailers offer varying profit margins and demand levels
- **Bulk Discounts**: Larger wholesale purchases offer better unit prices

#### Customer System
- **Order Types**: Individual requests, bulk orders, rush orders
- **Reputation**: Customer satisfaction affects future order frequency and pricing
- **Deadlines**: Time-sensitive orders offer higher profits but risk penalties
- **Negotiation**: Counter-offers and pricing flexibility

#### Market Dynamics
- **Seasonal Demand**: Holiday periods increase certain card types' popularity
- **Retailer Relationships**: Better wholesale prices through volume purchasing
- **Competition**: AI competitors affect market prices and customer availability
- **Random Events**: Market crashes, new retailer partnerships, promotional opportunities

### Progression System

#### Business Growth
- **Week 1-2**: Local customers, basic inventory management
- **Week 3-4**: Bulk orders, multiple retailer relationships
- **Week 5-8**: Seasonal planning, competitive pricing strategies
- **Week 9+**: Market expansion, advanced logistics

#### Unlockable Features
- **Advanced Analytics**: Profit trend analysis, demand forecasting
- **Bulk Storage**: Increased inventory capacity
- **Express Processing**: Rush order capabilities
- **Retailer Partnerships**: Exclusive wholesale deals

## User Interface Design

### Screen Hierarchy

```
Main Menu
├── New Game
│   └── Game Setup
│       └── Dashboard (Main Hub)
│           ├── Market Screen
│           ├── Customer Orders
│           ├── Inventory Management
│           ├── Analytics Dashboard
│           └── Settings
├── Continue Game
├── Tutorial
└── Quit
```

### Navigation System
- **Primary Navigation**: Number keys (1-6) for quick screen access
- **Secondary Navigation**: Arrow keys for menu navigation
- **Universal Controls**: 
  - `Esc` - Return to previous screen/dashboard
  - `Enter` - Confirm selection
  - `Tab` - Cycle through interface elements
  - `Space` - Pause/unpause game
  - `?` - Context-sensitive help

### Screen Specifications

#### Dashboard Layout
```
┌─ Header: Company name, cash, reputation, day, time ─┐
├─ Quick Actions: 6 numbered menu options            ─┤
├─ Activity Feed: Recent transactions and events     ─┤
└─ Footer: Control hints and notifications          ─┘
```

#### Market Screen Features
- Real-time wholesale pricing
- Stock availability indicators
- Shopping cart functionality
- Bulk discount calculations
- Retailer relationship status

#### Customer Orders Features
- Priority-based order sorting
- Profit margin calculations
- Deadline tracking
- Accept/decline/counter-offer options
- Customer history and preferences

#### Inventory Management Features
- Expiration date warnings
- Market value comparisons
- Quick-sell options
- Storage utilization metrics
- Loss prevention alerts

## Technical Implementation

### Core Data Structures

#### Game State
```rust
struct GameState {
    player: Player,
    inventory: Inventory,
    market: Market,
    orders: OrderManager,
    time: GameTime,
    events: EventQueue,
}
```

#### Card Types
```rust
struct GiftCard {
    retailer: Retailer,
    denomination: u32,
    purchase_price: u32,
    purchase_date: Date,
    expiration_date: Date,
}
```

#### Customer Orders
```rust
struct CustomerOrder {
    id: OrderId,
    customer: Customer,
    requested_cards: Vec<CardRequest>,
    offered_price: u32,
    deadline: Date,
    priority: Priority,
}
```

### Game Balance

#### Difficulty Scaling
- **Easy Mode**: 30% profit margins, 90-day expirations, forgiving customers
- **Normal Mode**: 20% profit margins, 60-day expirations, standard market
- **Hard Mode**: 15% profit margins, 30-day expirations, demanding customers

#### Economic Balance
- **Wholesale Costs**: 70-85% of retail value
- **Customer Offers**: 105-130% of retail value
- **Expiration Penalty**: 100% loss of expired inventory
- **Reputation Impact**: ±5% pricing flexibility per reputation level

## Success Metrics

### Win Conditions
- **Profit Goal**: Reach $50,000 net worth
- **Reputation Goal**: Achieve 5-star customer rating
- **Time Challenge**: Survive 365 game days
- **Market Share**: Control 25% of local gift card market

### Failure Conditions
- **Bankruptcy**: Negative cash flow with no sellable inventory
- **Reputation Loss**: Customer rating below 2 stars for 30 days
- **Inventory Crisis**: 50%+ of inventory expired in single month

## Development Phases

### Phase 1: Core Systems
- Basic TUI framework and navigation
- Inventory management system
- Simple market interactions
- Basic customer orders

### Phase 2: Game Logic
- Time progression and events
- Reputation system
- Economic balance tuning
- Save/load functionality

### Phase 3: Advanced Features
- Analytics dashboard
- Seasonal events
- Competition AI
- Achievement system

### Phase 4: Polish
- Tutorial system
- Sound effects (terminal bells)
- Advanced UI animations
- Performance optimization

## Risk Mitigation

### Technical Risks
- **TUI Complexity**: Start with simple layouts, iterate based on usability
- **Performance**: Optimize data structures for real-time updates
- **Cross-platform**: Test thoroughly on different terminal environments

### Design Risks
- **Complexity Creep**: Focus on core loop before adding features
- **Balance Issues**: Implement analytics early for data-driven tuning
- **User Experience**: Regular playtesting with target audience

## Future Expansion

### Potential Features
- **Multiplayer Mode**: Compete against other players
- **Campaign Mode**: Structured scenarios with specific challenges
- **Modding Support**: Custom retailers and card types
- **Mobile Port**: Adapt interface for mobile terminals

### Content Expansions
- **International Markets**: Global retailers and currencies
- **Digital Cards**: Cryptocurrency and digital gift cards
- **B2B Sales**: Corporate bulk orders and contracts
- **Franchise System**: Multiple store locations