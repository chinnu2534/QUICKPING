# 👤 User Guide - Finding and Using New Features

## 🔍 Where to Find Everything

### 1. Message Search
**Location**: Top of the left sidebar, below "Smart Highlights" button

```
┌─────────────────────────┐
│  [✨ Smart Highlights]  │
├─────────────────────────┤
│  [🔍 Search messages..] │  ← HERE!
│  [Search Button]        │
└─────────────────────────┘
```

**Steps to Use**:
1. Look at the left sidebar
2. Find the search box (has a 🔍 icon in placeholder)
3. Type at least 3 characters
4. Results appear below automatically
5. Click any result to jump to that message

**What You'll See**:
- Search results box appears below search input
- Each result shows:
  - Username of sender
  - Message preview
  - Timestamp
- Up to 10 results displayed
- "No messages found" if nothing matches

---

### 2. Pin Messages
**Location**: Top-right corner of EVERY message (appears on hover)

```
Message Layout:
┌────────────────────────────────┐
│  Username • Timestamp          │
│  Message text here...      [📌][😊] ← HOVER HERE!
│                                │
└────────────────────────────────┘
```

**Steps to Use**:
1. **Move your mouse** over ANY message in the conversation
2. **Look at the top-right corner** of the message
3. You'll see **TWO round buttons appear**:
   - 📌 = Pin button (left one)
   - 😊 = Reaction button (right one)
4. **Click the 📌 button** to pin
5. Message header will show "📌" when pinned
6. **Click 📌 again** to unpin

**Tips**:
- Buttons have white background with shadow
- They fade in when you hover
- Pin button turns yellow when message is pinned
- You'll see a notification when pinning/unpinning

---

### 3. User Online Status
**Location**: Next to each contact name in the contacts list

```
Contact Item Layout:
┌────────────────────────┐
│ ● Username         [✨]│  ← Green/Gray dot here!
└────────────────────────┘
```

**What the Colors Mean**:
- **🟢 Green dot with glow** = User is ONLINE
- **⚫ Gray dot** = User is OFFLINE

**Where to Look**:
1. Open your contacts list (left sidebar)
2. Look at the **left side** of each username
3. You'll see a small colored dot **before** the name

**Current Behavior**:
- Status is shown randomly for demonstration
- In production, this would show real online/offline status

---

### 4. React to Messages (Fixed Position)
**Location**: Same place as pin button (top-right of message on hover)

```
When You Hover:
┌────────────────────────────────┐
│  Username • Timestamp          │
│  Message text here...      [📌][😊] ← Click 😊
│                                │
└────────────────────────────────┘

Emoji Picker Appears:
┌──────────────────────┐
│  👍 ❤️ 😂 😢 😡    │ ← Choose one
└──────────────────────┘
```

**Steps to Use**:
1. Hover over any message
2. Click the **😊 button** (right button in top-right)
3. **Emoji picker appears** near the message
4. Click any emoji: 👍 ❤️ 😂 😢 😡
5. Your reaction appears below the message

**New Behavior** (Fixed!):
- Picker stays visible even when scrolling
- Always appears near the message
- Never goes off-screen
- Automatically positions above or below message
- Closes when you click outside

---

### 5. Typing Indicators (Ready, Not Active Yet)
**Location**: Bottom of messages area (when active)

```
Messages Area:
┌────────────────────────────────┐
│  Your conversation messages... │
│                                │
│  ● ● ●  ← Typing indicator     │
└────────────────────────────────┘
```

**Current Status**:
- Visual component is ready
- Animation is implemented
- NOT currently working (needs backend)

**Will Show**: When someone is typing a message to you

---

## 📱 Visual Walkthrough

### Finding Pin Button - Step by Step

**BEFORE hovering** (buttons hidden):
```
┌────────────────────────────────┐
│  John • 10:30 AM               │
│  Hey, how are you?             │
│  ❤️ 2                          │
└────────────────────────────────┘
```

**AFTER hovering** (buttons appear):
```
┌────────────────────────────────┐
│  John • 10:30 AM          [📌][😊]  ← THESE APPEAR!
│  Hey, how are you?             │
│  ❤️ 2                          │
└────────────────────────────────┘
```

**AFTER pinning** (shows pin indicator):
```
┌────────────────────────────────┐
│  📌 John • 10:30 AM       [📌][😊]  ← Pin in header
│  Hey, how are you?             │    + Button is yellow
│  ❤️ 2                          │
└────────────────────────────────┘
```

---

## 🎯 Common Questions

### Q: Why can't I see the pin button?
**A**: You need to **HOVER your mouse** over the message. The buttons only appear when you move your cursor over the message.

### Q: The emoji picker disappeared when I scrolled!
**A**: This was a bug and is now FIXED! The picker should stay visible even when scrolling.

### Q: How do I know if search is working?
**A**: Type at least 3 characters in the search box. Results should appear immediately below the search input.

### Q: Are user statuses real?
**A**: Currently they're random/demo. In production, they would show real online/offline status from the backend.

### Q: Can I search across all conversations?
**A**: Currently search only works in the active conversation. This is by design to keep results relevant.

### Q: Do pinned messages save?
**A**: Currently pins are session-based (lost on refresh). For permanent pins, backend database integration is needed.

---

## 🔍 Troubleshooting

### Pin Button Not Appearing:
1. **Make sure you're hovering** - Move mouse over the message
2. **Wait a moment** - Buttons fade in smoothly
3. **Check message area** - Look at top-right corner
4. **Try different message** - Scroll and try another message

### Search Not Working:
1. **Type more characters** - Need at least 3 characters
2. **Send some messages first** - Can't search empty conversation
3. **Check you're in a conversation** - Must have selected a contact
4. **Clear and try again** - Clear search box and retype

### Emoji Picker Won't Open:
1. **Hover over message first** - Buttons must be visible
2. **Click the 😊 button** - Right button, not left (📌)
3. **Wait for picker to appear** - Takes a moment
4. **Click outside to close** - If picker is stuck, click elsewhere

### Status Dots Not Visible:
1. **Look left of username** - Dots are BEFORE the name
2. **Check contacts list** - Must be in the sidebar contacts area
3. **Look for colored dots** - Green or gray, small circles
4. **Scroll contacts list** - Might need to scroll to see all

---

## 💡 Pro Tips

### Search Like a Pro:
- Type distinctive words for better results
- Use partial words (e.g., "meet" finds "meeting")
- Case doesn't matter (search is case-insensitive)
- Click the highlighted message to jump right to it

### Pinning Strategy:
- Pin important dates and times
- Pin decisions made in the conversation
- Pin meeting links or important information
- Don't pin too many (makes them less useful)

### Using Reactions:
- React instead of typing "ok" or "thanks"
- Use ❤️ for support or agreement
- Use 😂 for funny messages
- Use 👍 for acknowledgment

---

## 🎨 Visual Reference

### Button Appearance:
- **Size**: 32x32 pixels (easy to click)
- **Shape**: Perfect circles
- **Color**: White background with shadow
- **Hover**: Buttons appear with smooth fade-in
- **Active**: Pin button turns yellow when pinned

### Status Indicators:
- **Size**: 8x8 pixels (small dot)
- **Online**: Bright green with glow effect
- **Offline**: Gray/neutral color
- **Position**: Left of username, aligned vertically

### Search Results:
- **Background**: White boxes
- **Border**: Light gray separation
- **Hover**: Light gray background on hover
- **Layout**: Username (bold), message text, timestamp

---

## 🚀 Quick Start Checklist

After opening the app:

- [ ] Look for search box at top of sidebar
- [ ] Hover over a message to see pin/reaction buttons
- [ ] Check contacts for colored status dots
- [ ] Try pinning a message
- [ ] Try searching for a message
- [ ] Try reacting to a message

**If you see all these, everything is working!** ✅

---

## 📞 Still Need Help?

### Common Issues Solutions:

**"I really can't see the pin button"**
→ The button appears in the **TOP-RIGHT corner** of the message when you **HOVER**. It's not at the bottom. Try hovering slowly over a message and watch the top-right corner.

**"Emojis still go weird when I scroll"**
→ Make sure you're using the updated version. The fix changes the picker to stay in place even when scrolling.

**"Search shows no results but I know messages exist"**
→ Search only works in the current conversation. Make sure you're in the right conversation and have sent some messages there.

**"Status dots aren't changing"**
→ Currently they're random for demo. Real status tracking needs backend integration.

---

**Remember**: Most features appear on **HOVER**, so move your mouse around! 🖱️
