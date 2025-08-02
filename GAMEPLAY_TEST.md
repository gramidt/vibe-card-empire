# Gift Card Empire - Gameplay Test Guide

## Current Gameplay Features ✅

### 🎮 **Core Game Loop**
1. **Start Game**: Main menu → New Game → Dashboard
2. **Time Progression**: Game time advances automatically (10 minutes every 3 seconds)
3. **Purchase Cards**: Navigate to Market (press 1 or arrow keys + Enter)
4. **Manage Orders**: View customer requests (press 2)
5. **Pause/Resume**: Press Spacebar to pause time
6. **Navigation**: Use arrow keys, number keys (1-6), Enter, Esc

### 💰 **Economic System**
- **Starting Capital**: $5,000
- **Market Prices**: 
  - Amazon $25 cards cost $20 (25% markup potential)
  - Starbucks $10 cards cost $8 (25% markup potential)
  - Target $50 cards cost $42 (19% markup potential)
  - iTunes $15 cards cost $12 (25% markup potential)
  - Walmart $20 cards cost $17 (18% markup potential)

### ⏰ **Time Management**
- **Game Speed**: 10 minutes per 3 real seconds
- **Daily Events**: Inventory ages, expired cards removed, new orders appear
- **Card Expiration**: 30-90 days from purchase
- **Order Deadlines**: 2-6 days to fulfill

### 📋 **Customer Orders**
- **Dynamic Generation**: New orders appear every few days
- **Priority Levels**: 🔴 High, 🟡 Medium, 🟢 Low (based on profitability)
- **Reputation Effect**: Higher reputation = better customer offers
- **Order Aging**: Deadlines decrease daily, expired orders removed

## 🧪 **Testing Scenarios**

### **Scenario 1: Basic Purchase Flow**
1. Start new game
2. Go to Market (press 1)
3. Buy a cheap card (Starbucks $10 for $8)
4. Check activity feed shows purchase
5. Verify cash decreased from $5,000 to $4,992

### **Scenario 2: Time Progression**
1. Let game run for 30+ seconds
2. Watch time advance in header (should show progression)
3. Observe new customer orders appearing in activity feed
4. Test pause (Spacebar) - time should stop, resume should work

### **Scenario 3: Order Management**
1. Go to Orders screen (press 2)
2. See list of customer requests with priorities
3. Navigate with arrow keys to highlight different orders
4. Note order details: customer, item, quantity, deadline, profit potential

### **Scenario 4: Daily Events**
1. Wait for day to advance (or modify game speed for faster testing)
2. Watch for "Day X begins" message in activity feed
3. Observe inventory aging (cards get closer to expiration)
4. See expired orders disappear from orders list

### **Scenario 5: Economic Strategy**
1. Buy multiple cards of same type - should stack in inventory
2. Check different retailers for best profit margins
3. Monitor cash flow and spending decisions
4. Try to buy when insufficient funds - should show error message

## 🎯 **Expected Player Experience**

### **Early Game (Days 1-3)**
- Learning market prices and profit margins
- Building initial inventory
- Understanding time pressure
- Seeing first customer orders appear

### **Mid Game (Days 4-10)**
- Balancing purchase timing with order deadlines
- Managing cash flow as orders appear
- Dealing with inventory expiration risk
- Strategic pause usage for planning

### **Challenges to Notice**
- **Cash Management**: Can't buy everything at once
- **Time Pressure**: Need to act before orders expire
- **Expiration Risk**: Cards you buy might expire before selling
- **Profit Optimization**: Higher reputation = better customer offers

## 🐛 **Known Limitations (Coming Soon)**
- Can't actually fulfill customer orders yet (Accept button not implemented)
- No inventory selling mechanism
- Reputation doesn't change based on performance
- No save/load functionality
- No win/lose conditions

## 🏆 **Success Indicators**
The game is working well if you experience:
- ✅ Smooth time progression with visible changes
- ✅ Satisfying purchase decisions with immediate feedback
- ✅ Tension between spending money and waiting for orders
- ✅ Clear information display and easy navigation
- ✅ Engaging "just one more purchase" feeling

## 🛠 **Quick Fixes for Testing**
If you want to test faster:
- Edit `game_speed: Duration::from_secs(1)` in `main.rs` for faster time
- Modify `advance_time(30)` for bigger time jumps
- Adjust starting cash for different economic scenarios

The core gameplay loop is fully functional and provides a solid foundation for the remaining features!