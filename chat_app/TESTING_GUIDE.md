# 🧪 Testing Guide - Verify All Fixes

## Quick Test (5 Minutes)

Follow these steps to verify everything is working:

### ✅ Test 1: Search Messages (2 min)

1. **Start the app**: `cargo run`
2. **Login** with your account
3. **Select a contact** and send a few messages
4. **Find the search box**:
   - Look at the top of the left sidebar
   - Should see a text input with "🔍 Search messages..."
5. **Type in the search box**:
   - Type at least 3 characters from a message you sent
   - Results should appear immediately below
6. **Click a result**:
   - The conversation should scroll to that message
   - Message should highlight in yellow for 2 seconds

**Expected Result**: ✅ Search finds messages and jumps to them

**If it doesn't work**:
- Make sure you typed at least 3 characters
- Verify you're in an active conversation
- Check browser console for errors (F12)

---

### ✅ Test 2: Pin Messages (2 min)

1. **Open a conversation** with some messages
2. **Hover your mouse** over ANY message slowly
3. **Look at the TOP-RIGHT corner** of the message
4. **You should see 2 buttons appear**:
   ```
   [📌] [😊]
   ```
5. **Click the left button (📌)**
6. **Check the message header** - should now show "📌 Username • Time"
7. **Click the pin button again** to unpin

**Expected Result**: ✅ Pin button appears on hover, pins/unpins messages

**If you can't see buttons**:
- HOVER SLOWLY over the message
- Look at TOP-RIGHT corner (not bottom)
- Buttons should fade in within 0.2 seconds
- Make sure your mouse is actually over the message

---

### ✅ Test 3: User Status (1 min)

1. **Look at your contacts list** (left sidebar)
2. **Find the contacts section**
3. **Look at each contact name**
4. **BEFORE the username**, you should see a small colored dot:
   - 🟢 Green dot = Online
   - ⚫ Gray dot = Offline

**Expected Result**: ✅ Every contact has a colored dot before their name

**If you don't see dots**:
- Look LEFT of the username (not right)
- Dots are small (8x8 pixels)
- Should be right before the first letter of name
- Try scrolling the contacts list

---

### ✅ Test 4: Emoji Reactions (2 min)

**Part A: Opening the Picker**
1. **Hover over a message**
2. **Click the right button (😊)** in the top-right corner
3. **Emoji picker should appear** near the message with 5 emojis:
   ```
   👍 ❤️ 😂 😢 😡
   ```

**Part B: Test Fixed Positioning**
4. **Don't close the picker yet**
5. **Scroll the conversation** up and down
6. **The picker should stay in view** (NOT move with the scroll)
7. **Click an emoji** to react
8. **Reaction appears** below the message

**Part C: Test Auto-Positioning**
9. **Try messages at different positions**:
   - Message at top of screen
   - Message at bottom of screen
   - Message at left edge
   - Message at right edge
10. **Picker should always stay visible** and not go off-screen

**Expected Result**:
✅ Picker appears near message
✅ Stays visible when scrolling
✅ Never goes off-screen
✅ Closes when clicking outside

**If picker moves when scrolling**:
- You may have an old version
- Refresh the page (Ctrl+R or Cmd+R)
- Clear browser cache if needed

---

## Detailed Testing (15 Minutes)

### Test Suite 1: Search Functionality

#### Test 1.1: Basic Search
```
Steps:
1. Send messages: "Hello", "How are you?", "Meeting at 3pm"
2. Search for "meet"
3. Verify result shows "Meeting at 3pm"
4. Click result
5. Verify message highlights

Expected: ✅ Search is case-insensitive and finds partial matches
```

#### Test 1.2: Real-time Search
```
Steps:
1. Type "H" - no results yet
2. Type "He" - no results yet
3. Type "Hel" - results appear
4. Type "Hell" - results refine

Expected: ✅ Search activates after 3 characters
```

#### Test 1.3: No Results
```
Steps:
1. Search for "xyz123abc" (something that doesn't exist)
2. Verify "No messages found" appears

Expected: ✅ Clear feedback when no matches
```

#### Test 1.4: Clear Search
```
Steps:
1. Perform a search with results
2. Clear the search box (delete all text)
3. Verify results disappear

Expected: ✅ Results hide when search is empty
```

#### Test 1.5: Conversation Switch
```
Steps:
1. Search in Conversation A
2. Switch to Conversation B
3. Verify search results cleared
4. Search works in Conversation B

Expected: ✅ Search is conversation-specific
```

---

### Test Suite 2: Pin Message Functionality

#### Test 2.1: Pin Button Visibility
```
Steps:
1. Don't hover over message - no buttons
2. Hover over message - buttons appear
3. Move mouse away - buttons fade out
4. Repeat on 5 different messages

Expected: ✅ Buttons consistently appear on hover
```

#### Test 2.2: Pin Action
```
Steps:
1. Pin a message
2. Verify notification appears
3. Verify header shows "📌"
4. Verify pin button turns yellow
5. Verify left border is yellow

Expected: ✅ Clear visual feedback for pinned state
```

#### Test 2.3: Unpin Action
```
Steps:
1. Unpin a previously pinned message
2. Verify notification appears
3. Verify "📌" removed from header
4. Verify pin button returns to white
5. Verify left border returns to normal

Expected: ✅ Pinned state toggles correctly
```

#### Test 2.4: Multiple Pins
```
Steps:
1. Pin 3 different messages
2. Verify all show pinned indicators
3. Unpin one
4. Verify others remain pinned

Expected: ✅ Can pin multiple messages independently
```

#### Test 2.5: Pin Persistence
```
Steps:
1. Pin a message
2. Switch to another conversation
3. Return to original conversation
4. Check if message is still pinned

Expected: ⚠️ Currently NOT persistent (by design)
Note: Pins are session-based currently
```

---

### Test Suite 3: User Status Indicators

#### Test 3.1: Status Visibility
```
Steps:
1. Count all contacts
2. Verify each has a status dot
3. Check dot is before username
4. Verify some are green, some are gray

Expected: ✅ All contacts show status
```

#### Test 3.2: Status Colors
```
Steps:
1. Find a contact with green dot
2. Verify dot is bright green
3. Verify dot has a subtle glow
4. Find a contact with gray dot
5. Verify dot is neutral gray

Expected: ✅ Colors are distinct and visible
```

#### Test 3.3: Status Position
```
Steps:
1. Look at contact item structure
2. Verify order: [dot] [username] [✨ button]
3. Verify dot aligns vertically with username

Expected: ✅ Consistent layout across all contacts
```

---

### Test Suite 4: Emoji Picker Positioning

#### Test 4.1: Basic Opening
```
Steps:
1. Click reaction button on middle message
2. Verify picker appears
3. Verify picker has 5 emojis
4. Verify border is visible

Expected: ✅ Picker opens reliably
```

#### Test 4.2: Scroll Test (Critical Fix)
```
Steps:
1. Open emoji picker on a message
2. Scroll conversation down
3. Verify picker DOES NOT move
4. Scroll conversation up
5. Verify picker STILL does not move
6. Picker should stay in fixed position

Expected: ✅ Picker stays in viewport when scrolling
```

#### Test 4.3: Top Message
```
Steps:
1. Scroll to top of conversation
2. Open picker on topmost message
3. Verify picker appears (should be below message)
4. Verify picker is fully visible

Expected: ✅ Picker positions below when no space above
```

#### Test 4.4: Bottom Message
```
Steps:
1. Scroll to bottom of conversation
2. Open picker on bottom message
3. Verify picker appears (should be above message)
4. Verify picker is fully visible

Expected: ✅ Picker positions above when space available
```

#### Test 4.5: Left Edge
```
Steps:
1. Open picker on a sent message (right side)
2. Verify picker doesn't go off-screen left
3. Picker should be visible within viewport

Expected: ✅ Picker stays within horizontal bounds
```

#### Test 4.6: Right Edge
```
Steps:
1. Open picker on a received message (left side)
2. Verify picker doesn't go off-screen right
3. Picker should be visible within viewport

Expected: ✅ Picker stays within horizontal bounds
```

#### Test 4.7: Click Outside
```
Steps:
1. Open emoji picker
2. Click somewhere else on the page
3. Verify picker closes

Expected: ✅ Picker closes on outside click
```

#### Test 4.8: Select Emoji
```
Steps:
1. Open picker
2. Click an emoji (e.g., 👍)
3. Verify picker closes
4. Verify reaction appears below message

Expected: ✅ Reaction works and picker closes
```

---

### Test Suite 5: Cross-Browser Testing

#### Test Each Browser:
- [ ] Chrome/Chromium
- [ ] Firefox
- [ ] Safari (Mac only)
- [ ] Edge
- [ ] Mobile Chrome (if available)
- [ ] Mobile Safari (if available)

#### For Each Browser Test:
1. Search functionality
2. Pin button visibility and action
3. Status indicators
4. Emoji picker positioning
5. Emoji picker scroll behavior

---

## Mobile Testing (If Applicable)

### Mobile-Specific Tests:

#### Test M1: Touch Hover Behavior
```
On mobile, "hover" doesn't exist, so:
1. Tap on a message
2. Check if buttons appear
3. If not, buttons may need tap-to-show on mobile

Note: This may need adjustment for mobile
```

#### Test M2: Emoji Picker on Mobile
```
Steps:
1. Open emoji picker
2. Verify picker is large enough for touch
3. Verify emojis are easy to tap
4. Test on small screen (iPhone SE size)

Expected: ✅ Usable on small screens
```

#### Test M3: Search on Mobile
```
Steps:
1. Open mobile keyboard
2. Type in search
3. Verify results display properly
4. Verify keyboard doesn't cover results

Expected: ✅ Search works with mobile keyboard
```

---

## Performance Testing

### Test P1: Search Performance
```
Steps:
1. Have conversation with 100+ messages
2. Perform search
3. Measure response time

Expected: ✅ Results appear within 100ms
```

### Test P2: Emoji Picker Performance
```
Steps:
1. Open and close picker 10 times rapidly
2. Check for lag or stuttering

Expected: ✅ Smooth, no lag
```

### Test P3: Status Indicators Load
```
Steps:
1. Have 50+ contacts
2. Observe initial load
3. Check if all dots appear quickly

Expected: ✅ All indicators load instantly
```

---

## Edge Case Testing

### Edge Case 1: Very Long Message
```
Steps:
1. Send a very long message (500+ characters)
2. Hover over it
3. Verify buttons still appear correctly
4. Open emoji picker
5. Verify picker positions correctly

Expected: ✅ Works with long messages
```

### Edge Case 2: First Message in Conversation
```
Steps:
1. Start a new conversation
2. Send first message
3. Test all features on this message

Expected: ✅ All features work on first message
```

### Edge Case 3: Rapid Actions
```
Steps:
1. Open/close emoji picker rapidly (10x)
2. Pin/unpin rapidly (10x)
3. Search and clear rapidly (10x)

Expected: ✅ No errors, UI remains stable
```

### Edge Case 4: Special Characters in Search
```
Steps:
1. Send message with emojis: "Hello 👋 World"
2. Search for "Hello"
3. Verify it finds the message

Expected: ✅ Search handles special characters
```

---

## Regression Testing

### Verify Old Features Still Work:

- [ ] Login/Registration
- [ ] Send messages
- [ ] Receive messages
- [ ] Message history loads
- [ ] Groups work
- [ ] Games work
- [ ] Polls work
- [ ] Smart highlights work
- [ ] AI assistant works

---

## Automated Testing Checklist

```javascript
// Pseudo-code for automated tests

test('Search shows results after 3 characters', () => {
  // Type 2 chars - no results
  // Type 3 chars - results appear
  // Verify results count > 0
});

test('Pin button appears on hover', () => {
  // Hover over message
  // Wait 300ms
  // Check if .message-actions has opacity > 0
});

test('Emoji picker stays fixed when scrolling', () => {
  // Open picker
  // Get picker position
  // Scroll page
  // Get picker position again
  // Verify position is fixed, not absolute
});

test('Status dots present for all contacts', () => {
  // Get all contact items
  // For each contact
  //   Verify .user-status element exists
});
```

---

## Bug Report Template

If you find an issue:

```
### Bug Description
[What's wrong?]

### Steps to Reproduce
1.
2.
3.

### Expected Behavior
[What should happen?]

### Actual Behavior
[What actually happens?]

### Browser/Device
- Browser:
- Version:
- OS:
- Screen size:

### Screenshots
[If applicable]

### Console Errors
[Any errors in browser console? F12 to check]
```

---

## Success Criteria

All tests pass when:

- ✅ Search finds messages in current conversation
- ✅ Search highlights clicked results
- ✅ Pin button visible on message hover
- ✅ Pin action shows clear visual feedback
- ✅ Status dots appear for all contacts
- ✅ Emoji picker opens near message
- ✅ Emoji picker STAYS FIXED when scrolling
- ✅ Emoji picker never goes off-screen
- ✅ All features work across browsers
- ✅ No console errors
- ✅ No visual glitches

---

## Final Verification

Before marking as complete:

1. ✅ Run Quick Test (5 min)
2. ✅ Run Detailed Testing (15 min)
3. ✅ Test in at least 2 browsers
4. ✅ Check for console errors
5. ✅ Verify regression tests pass
6. ✅ Check mobile if applicable
7. ✅ Document any issues found

**If all checks pass: APPLICATION IS VERIFIED** ✅

---

**Happy Testing!** 🧪
