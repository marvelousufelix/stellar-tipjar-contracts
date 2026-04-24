# TipJar Contract — Events Reference

All events are emitted via `env.events().publish(topics, data)` and are queryable from Stellar's event streaming API.

---

## `tip`

Emitted when a plain tip is successfully transferred into escrow.

**Topics**

| Position | Value | Type |
|---|---|---|
| 0 | `"tip"` | `Symbol` |
| 1 | `creator` | `Address` |

**Data**

| Field | Type | Description |
|---|---|---|
| `sender` | `Address` | The address that sent the tip |
| `amount` | `i128` | Token amount transferred |

**Example (conceptual)**
```
topics: ["tip", "GCREATOR..."]
data:   ["GSENDER...", 250]
```

---

## `tip_msg`

Emitted when a tip is sent with an attached message via `tip_with_message`.

**Topics**

| Position | Value | Type |
|---|---|---|
| 0 | `"tip_msg"` | `Symbol` |
| 1 | `creator` | `Address` |

**Data**

| Field | Type | Description |
|---|---|---|
| `sender` | `Address` | The address that sent the tip |
| `amount` | `i128` | Token amount transferred |
| `message` | `String` | The attached message text |
| `metadata` | `Map<String, String>` | Arbitrary key-value metadata |

**Example (conceptual)**
```
topics: ["tip_msg", "GCREATOR..."]
data:   ["GSENDER...", 500, "Great content!", {"platform": "web"}]
```

---

## `withdraw`

Emitted when a creator successfully withdraws their escrowed balance.

**Topics**

| Position | Value | Type |
|---|---|---|
| 0 | `"withdraw"` | `Symbol` |
| 1 | `creator` | `Address` |

**Data**

| Field | Type | Description |
|---|---|---|
| `amount` | `i128` | Total amount withdrawn |

**Example (conceptual)**
```
topics: ["withdraw", "GCREATOR..."]
data:   400
```

---

## `delegate`

Emitted when a creator grants withdrawal authorization to a delegate.

**Topics**

| Position | Value | Type |
|---|---|---|
| 0 | `"delegate"` | `Symbol` |
| 1 | `creator` | `Address` |

**Data**

| Field | Type | Description |
|---|---|---|
| `delegate` | `Address` | The authorized delegate address |
| `max_amount` | `i128` | Lifetime withdrawal cap for the delegation |
| `expires_at` | `u64` | Expiry timestamp for the delegation |

**Example (conceptual)**
```
topics: ["delegate", "GCREATOR..."]
data:   ["GDELEGATE...", 300, 1_800_000_100]
```

---

## `delegate_withdraw`\n\nEmitted when a delegate successfully withdraws on behalf of a creator.\n\n**Topics**\n\n| Position | Value | Type |\n|---|---|---|\n| 0 | `\"del_wdr\"` | `Symbol` |\n| 1 | `creator` | `Address` |\n\n**Data**\n\n| Field | Type | Description |\n|---|---|---|\n| `delegate` | `Address` | The delegate that withdrew funds |\n| `amount` | `i128` | Amount withdrawn |\n| `token` | `Address` | The token asset withdrawn |\n\n**Example (conceptual)**\n```\ntopics: [\"del_wdr\", \"GCREATOR...\"]\ndata:   [\"GDELEGATE...\", 200, \"GTOKEN...\"]\n```\n\n---\n\n## `delegate_revoked`\n\nEmitted when a creator revokes a delegation.\n\n**Topics**\n\n| Position | Value | Type |\n|---|---|---|\n| 0 | `\"del_rev\"` | `Symbol` |\n| 1 | `creator` | `Address` |\n\n**Data**\n\n| Field | Type | Description |\n|---|---|---|\n| `delegate` | `Address` | The delegate that was revoked |\n\n**Example (conceptual)**\n```\ntopics: [\"del_rev\", \"GCREATOR...\"]\ndata:   [\"GDELEGATE...\"]\n```\n\n---\n\n## `tip_expired`\n
Emitted when an unclaimed time-locked tip is refunded after its expiration window.

**Topics**

| Position | Value | Type |
|---|---|---|
| 0 | `"tip_expired"` | `Symbol` |
| 1 | `creator` | `Address` |

**Data**

| Field | Type | Description |
|---|---|---|
| `sender` | `Address` | The original tipper's address |
| `amount` | `i128` | Refunded token amount |
| `expires_at` | `u64` | Expiry timestamp that triggered the refund |
| `lock_id` | `u64` | Identifier for the refunded time-lock |

**Example (conceptual)**
```
topics: ["tip_expired", "GCREATOR..."]
data:   ["GSENDER...", 500, 1_800_000_100, 0]
```

---

## Querying Events

Use the Stellar CLI to stream or query events from a deployed contract:

```bash
stellar events \
  --network testnet \
  --contract-id <CONTRACT_ID> \
  --start-ledger <LEDGER>
```

Filter by topic to watch only tip events:

```bash
stellar events \
  --network testnet \
  --contract-id <CONTRACT_ID> \
  --topic1 tip
```
