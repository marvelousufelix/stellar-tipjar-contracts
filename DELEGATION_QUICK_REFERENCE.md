# Delegation System - Quick Reference

## Usage Examples

### 1. Authorize a Delegate (Creator)
```rust
// Creator authorizes account B to withdraw up to 1000 units for 7 days
client.delegate_withdrawal(
    &creator_account,      // Creator (must authorize)
    &delegate_account,     // Delegate address
    &1000i128,             // Max amount
    &604800u64             // 7 days in seconds (7 * 24 * 3600)
);
// Event: ("delegate", creator) → (delegate, 1000, expires_at)
```

### 2. Withdraw as Delegate
```rust
// Delegate withdraws on behalf of creator
client.withdraw_as_delegate(
    &delegate_account,     // Delegate (must authorize)
    &creator_account,      // Creator
    &token_address,        // Token to withdraw
    &500i128               // Amount to withdraw
);
// Event: ("del_wdr", creator) → (delegate, 500, token)
```

### 3. Check Delegation Status
```rust
// Check if delegation exists and is valid
match client.get_delegation(&creator, &delegate) {
    Some(delegation) => {
        println!("Max: {}", delegation.max_amount);
        println!("Used: {}", delegation.used_amount);
        println!("Active: {}", delegation.active);
        println!("Expires: {}", delegation.expires_at);
    }
    None => println!("No delegation found"),
}
```

### 4. List Active Delegates
```rust
let delegates = client.get_delegates(&creator);
for delegate in delegates {
    println!("Delegate: {}", delegate);
}
```

### 5. Revoke Delegation
```rust
client.revoke_delegation(&creator, &delegate);
// Event: ("del_rev", creator) → (delegate,)
```

## Event Symbols

| Symbol | Full Name | Emitted By |
|--------|-----------|-----------|
| `"delegate"` | Delegation authorized | `delegate_withdrawal` |
| `"del_wdr"` | Delegate withdrawal | `withdraw_as_delegate` |
| `"del_rev"` | Delegation revoked | `revoke_delegation` |

## Authorization Requirements

| Operation | Signer Required | Reason |
|-----------|-----------------|--------|
| `delegate_withdrawal` | Creator | Only creator can grant authority |
| `withdraw_as_delegate` | Delegate | Delegate must authenticate withdrawal |
| `revoke_delegation` | Creator | Only creator can revoke |
| `get_delegation` | None | Read-only query |
| `get_delegates` | None | Read-only query |
| `get_delegation_history` | None | Read-only query |

## State Machine

```
┌─────────────────────┐
│  No Delegation      │
└──────────┬──────────┘
           │
           │ delegate_withdrawal()
           ▼
┌─────────────────────┐
│  Active Delegation  │ ◄─ Check: used_amount < max_amount
│  active = true      │     Check: current_time < expires_at
└──────┬──────┬───────┘
       │      │
       │      │ time passes beyond expires_at
       │      │ OR used_amount reaches max_amount
       │      ▼
       │    ┌──────────────────────┐
       │    │ Deactivated          │
       │    │ active = false       │
       │    │ (auto-deactivation)  │
       │    └──────────────────────┘
       │
       │ revoke_delegation()
       ▼
┌─────────────────────┐
│ Revoked Delegation  │ (stored in history)
│ active = false      │
└─────────────────────┘
```

## Error Scenarios

### Error: DelegationNotFound
- Cause: No delegation exists for the creator/delegate pair
- Fix: Call `delegate_withdrawal()` first to create delegation

### Error: DelegationExpired
- Cause: Current timestamp > delegation.expires_at
- Fix: Creator must call `delegate_withdrawal()` with new expiry
- Note: Delegation auto-marks as inactive

### Error: DelegationInactive
- Cause: Delegation was revoked or auto-deactivated
- Fix: Creator must authorize a new delegation

### Error: DelegationLimitExceeded
- Cause: `used_amount + withdraw_amount > max_amount`
- Fix: Withdraw smaller amount or request higher limit

### Error: InvalidDuration
- Cause: Duration == 0
- Fix: Specify duration > 0

### Error: InvalidAmount
- Cause: max_amount <= 0
- Fix: Specify max_amount > 0

## Delegation History

The `get_delegation_history()` returns all snapshots including:
- Original authorization (new delegation)
- Each withdrawal (updated used_amount)
- Expiration (auto-deactivation)
- Explicit revocation

Example history for a delegation:
```
[
  Delegation { used_amount: 0, active: true, ... },   // Created
  Delegation { used_amount: 100, active: true, ... },  // First withdrawal
  Delegation { used_amount: 200, active: false, ... }, // Revoked
]
```

## Gas Optimization Tips

1. **Check balance first**: Use `get_withdrawable_balance()` before calling `withdraw_as_delegate()`
2. **Batch queries**: Call `get_delegates()` once instead of looping n times
3. **Cache expiry**: Check `delegation.expires_at` before attempting withdrawal
4. **Reuse delegations**: Rather than creating/revoking often, extend duration once

## Common Workflows

### Team Tip Processing
```rust
// Creator delegates to accounting team member
client.delegate_withdrawal(&creator, &accountant, &10000i128, &2592000u64); // 30 days

// Later, team member collects tips
let balance = client.get_withdrawable_balance(&creator, &token);
if balance > 0 {
    client.withdraw_as_delegate(&accountant, &creator, &token, &balance);
}

// When engagement ends
client.revoke_delegation(&creator, &accountant);
```

### Temporary Authorization
```rust
// Creator grants limited authority for specific event
client.delegate_withdrawal(&creator, &event_handler, &500i128, &86400u64); // 24 hours

// Handler processes tips during event
client.withdraw_as_delegate(&event_handler, &creator, &token, &500i128);

// Automatically expires after 24 hours
```

---

**For detailed API documentation, see:** `docs/CONTRACT_SPEC.md`  
**For event details, see:** `docs/EVENTS.md`  
**For tests and examples, see:** `contracts/tipjar/tests/delegation_tests.rs`
