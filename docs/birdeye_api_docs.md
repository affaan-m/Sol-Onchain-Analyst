# Birdeye API Documentation

This document provides information about the Birdeye API.

## Token - Market Data

/defi/v3/token/market-data

Retrieves market data for a DeFi token (v3).

**Parameters:**

| Name                  | Type    | Required | Default     | Description                                                                                                                                                                                                                                                                                          |
| :-------------------- | :------ | :------- | :---------- | :--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `address`             | string  | Yes      |             | The address of the token to retrieve market data for                                                                                                                                                                                                                                                                                          |

**Example Request (cURL):**

```bash
curl --request GET \
     --url 'https://public-api.birdeye.so/defi/v3/token/market-data?address=So11111111111111111111111111111111111111112' \
     --header 'X-API-KEY: e218eef66dd64c3c9eaffc048daecfd4' \
     --header 'accept: application/json' \
     --header 'x-chain: solana'
```

**Response:**

```json
{
  "data": {
    "address": "So11111111111111111111111111111111111111112",
    "price": 169.42468902661997,
    "liquidity": 21152390225.973328,
    "supply": 594530296.631611,
    "total_supply": 594530296.631611,
    "circulating_supply": 488611763.26109004,
    "marketcap": 100728110623.71481,
    "fdv": 100728110623.71481,
    "circulating_marketcap": 82782896045.25864,
    "market_cap": 82782896045.25864
  },
  "success": true
}
```

---

## Token - Trade Data (Single)

/defi/v3/token/trade-data/single

Retrieves trade data for a single DeFi token (v3).

**Parameters:**

| Name                  | Type    | Required | Default     | Description                                                                                                                                                                                                                                                                                          |
| :-------------------- | :------ | :------- | :---------- | :--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `address`             | string  | Yes      |             | The address of the token to retrieve trade data for                                                                                                                                                                                                                                                                                          |

**Example Request (cURL):**

```bash
curl --request GET \
     --url 'https://public-api.birdeye.so/defi/v3/token/trade-data/single?address=So11111111111111111111111111111111111111112' \
     --header 'X-API-KEY: e218eef66dd64c3c9eaffc048daecfd4' \
     --header 'accept: application/json' \
     --header 'x-chain: solana'
```

**Response:**

```json
{
  "data": {
    "address": "So11111111111111111111111111111111111111112",
    "holder": 1306583,
    "market": 248711,
    "last_trade_unix_time": 1740168819,
    "last_trade_human_time": "2025-02-21T20:13:39",
    "price": 169.3068938959161,
    "history_30m_price": 169.09877678056895,
    "price_change_30m_percent": 0.12307428788630395,
    "history_1h_price": 170.87840598582463,
    "price_change_1h_percent": -0.9196668712130331,
    "history_2h_price": 171.7754043889939,
    "price_change_2h_percent": -1.437057011659103,
    "history_4h_price": 174.30006937251954,
    "price_change_4h_percent": -2.864700797067317,
    "history_6h_price": 179.13233242978072,
    "price_change_6h_percent": -5.485016803270938,
    "history_8h_price": 176.97954657637476,
    "price_change_8h_percent": -4.335332996882534,
    "history_12h_price": 176.99877129411328,
    "price_change_12h_percent": -4.345723612632227,
    "history_24h_price": 175.24286123836828,
    "price_change_24h_percent": -3.38728054341568,
    "unique_wallet_30m": 75548,
    "unique_wallet_history_30m": 72743,
    "unique_wallet_30m_change_percent": 3.8560411311053984,
    "unique_wallet_1h": 118590,
    "unique_wallet_history_1h": 119946,
    "unique_wallet_1h_change_percent": -1.1305087289280176,
    "unique_wallet_2h": 196705,
    "unique_wallet_history_2h": 196749,
    "unique_wallet_2h_change_percent": -0.022363519001367224,
    "unique_wallet_4h": 328852,
    "unique_wallet_history_4h": 324108,
    "unique_wallet_4h_change_percent": 1.4637096276549793,
    "unique_wallet_8h": 545116,
    "unique_wallet_history_8h": 495066,
    "unique_wallet_8h_change_percent": 10.109763142692085,
    "unique_wallet_24h": 1313801,
    "unique_wallet_history_24h": 1461935,
    "unique_wallet_24h_change_percent": -10.132735039519542,
    "trade_30m": 739449,
    "trade_history_30m": 748644,
    "trade_30m_change_percent": -1.2282206228861783,
    "sell_30m": 424326,
    "sell_history_30m": 422586,
    "sell_30m_change_percent": 0.4117505075889878,
    "buy_30m": 315123,
    "buy_history_30m": 326058,
    "buy_30m_change_percent": -3.353697808365383,
    "volume_30m": 621543.7980476739,
    "volume_30m_usd": 104974828.50743121,
    "volume_history_30m": 585269.7705942759,
    "volume_history_30m_usd": 99280522.68745488,
    "volume_30m_change_percent": 6.197830346946805,
    "volume_buy_30m": 316420.371212389,
    "volume_buy_30m_usd": 53445996.4543387,
    "volume_buy_history_30m": 292347.92663939495,
    "volume_buy_history_30m_usd": 49585378.41720483,
    "volume_buy_30m_change_percent": 8.234176602417618,
    "volume_sell_30m": 305123.4268352849,
    "volume_sell_30m_usd": 51528832.05309251,
    "volume_sell_history_30m": 292921.84395488095,
    "volume_sell_history_30m_usd": 49695144.27025006,
    "volume_sell_30m_change_percent": 4.165473873735202,
    "trade_1h": 1385862,
    "trade_history_1h": 1314580,
    "trade_1h_change_percent": 5.422416285049217,
    "sell_1h": 788925,
    "sell_history_1h": 762914,
    "sell_1h_change_percent": 3.409427537048737,
    "buy_1h": 596937,
    "buy_history_1h": 551666,
    "buy_1h_change_percent": 8.20623348185315,
    "volume_1h": 1128052.833893632,
    "volume_1h_usd": 190955210.78821248,
    "volume_history_1h": 966427.002982611,
    "volume_history_1h_usd": 165793473.74042892,
    "volume_1h_change_percent": 16.72405990439086,
    "volume_buy_1h": 565527.6869574629,
    "volume_buy_1h_usd": 95729452.39851624,
    "volume_buy_history_1h": 495955.1459618859,
    "volume_buy_history_1h_usd": 85090196.33163825,
    "volume_buy_1h_change_percent": 14.027990547541094,
    "volume_sell_1h": 562525.146936169,
    "volume_sell_1h_usd": 95225758.38969623,
    "volume_sell_history_1h": 470471.85702072503,
    "volume_sell_history_1h_usd": 80703277.40879066,
    "volume_sell_1h_change_percent": 19.566162894072725,
    "trade_2h": 2605151,
    "trade_history_2h": 2700998,
    "trade_2h_change_percent": -3.548577229601799,
    "sell_2h": 1495698,
    "sell_history_2h": 1505155,
    "sell_2h_change_percent": -0.6283073836249423,
    "buy_2h": 1109453,
    "buy_history_2h": 1195843,
    "buy_2h_change_percent": -7.224192473426696,
    "volume_2h": 2035313.756564898,
    "volume_2h_usd": 346638582.50251836,
    "volume_history_2h": 2590215.1025522994,
    "volume_history_2h_usd": 449448620.6706019,
    "volume_2h_change_percent": -21.42298318933523,
    "volume_buy_2h": 1034619.728362683,
    "volume_buy_2h_usd": 176229213.49061972,
    "volume_buy_history_2h": 1245683.5442151865,
    "volume_buy_history_2h_usd": 216090275.89209786,
    "volume_buy_2h_change_percent": -16.943614357969167,
    "volume_sell_2h": 1000694.0282022151,
    "volume_sell_2h_usd": 170409369.01189864,
    "volume_sell_history_2h": 1344531.5583371127,
    "volume_sell_history_2h_usd": 233358344.77850404,
    "volume_sell_2h_change_percent": -25.57303530756454,
    "trade_4h": 5431868,
    "trade_history_4h": 4882033,
    "trade_4h_change_percent": 11.262418750549209,
    "sell_4h": 3066637,
    "sell_history_4h": 2682698,
    "sell_4h_change_percent": 14.311674292074619,
    "buy_4h": 2365231,
    "buy_history_4h": 2199335,
    "buy_4h_change_percent": 7.543007318121159,
    "volume_4h": 4973086.600929389,
    "volume_4h_usd": 856402299.8813775,
    "volume_history_4h": 5592845.022419004,
    "volume_history_4h_usd": 987387729.7552037,
    "volume_4h_change_percent": -11.081272930061607,
    "volume_buy_4h": 2498817.022634697,
    "volume_buy_4h_usd": 430231995.6544323,
    "volume_buy_history_4h": 2830137.6637956523,
    "volume_buy_history_4h_usd": 499713309.0384833,
    "volume_buy_4h_change_percent": -11.70687367612369,
    "volume_sell_4h": 2474269.578294692,
    "volume_sell_4h_usd": 426170304.22694516,
    "volume_sell_history_4h": 2762707.358623352,
    "volume_sell_history_4h_usd": 487674420.7167204,
    "volume_sell_4h_change_percent": -10.44040294127958,
    "trade_8h": 9655648,
    "trade_history_8h": 7948679,
    "trade_8h_change_percent": 21.474876517217513,
    "sell_8h": 5395516,
    "sell_history_8h": 4500189,
    "sell_8h_change_percent": 19.895319952117564,
    "buy_8h": 4260132,
    "buy_history_8h": 3448490,
    "buy_8h_change_percent": 23.53615640468727,
    "volume_8h": 9553602.065115437,
    "volume_8h_usd": 1667622402.8046503,
    "volume_history_8h": 6428409.162792007,
    "volume_history_8h_usd": 1135365784.4775598,
    "volume_8h_change_percent": 48.615338930387644,
    "volume_buy_8h": 4814806.345671134,
    "volume_buy_8h_usd": 840537288.1722652,
    "volume_buy_history_8h": 3181553.2293998343,
    "volume_buy_history_8h_usd": 561905311.3742208,
    "volume_buy_8h_change_percent": 51.33508693737604,
    "volume_sell_8h": 4738795.719444304,
    "volume_sell_8h_usd": 827085114.632385,
    "volume_sell_history_8h": 3246855.933392173,
    "volume_sell_history_8h_usd": 573460473.103339,
    "volume_sell_8h_change_percent": 45.95029211824059,
    "trade_24h": 26958184,
    "trade_history_24h": 25551807,
    "trade_24h_change_percent": 5.5040216920862,
    "sell_24h": 15436562,
    "sell_history_24h": 14464945,
    "sell_24h_change_percent": 6.717045934153224,
    "buy_24h": 11521622,
    "buy_history_24h": 11086862,
    "buy_24h_change_percent": 3.9213981377237315,
    "volume_24h": 22925044.55517865,
    "volume_24h_usd": 4013947241.7719307,
    "volume_history_24h": 21220971.909134775,
    "volume_history_24h_usd": 3644897146.7027364,
    "volume_24h_change_percent": 8.03013478053915,
    "volume_buy_24h": 11430638.192880036,
    "volume_buy_24h_usd": 2001502491.499937,
    "volume_buy_history_24h": 10367432.597364351,
    "volume_buy_history_24h_usd": 1780645589.914619,
    "volume_buy_24h_change_percent": 10.255244830682356,
    "volume_sell_24h": 11494406.362298615,
    "volume_sell_24h_usd": 2012444750.2719934,
    "volume_sell_history_24h": 10853539.311770422,
    "volume_sell_history_24h_usd": 1864251556.7881174,
    "volume_sell_24h_change_percent": 5.904682630422565
  },
  "success": true
}
```

---

## Token - Metadata (Single)

/defi/v3/token/meta-data/single

Retrieves metadata for a single DeFi token (v3).

**Parameters:**

| Name                  | Type    | Required | Default     | Description                                                                                                                                                                                                                                                                                          |
| :-------------------- | :------ | :------- | :---------- | :--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `address`             | string  | Yes      |             | The address of the token to retrieve metadata for                                                                                                                                                                                                                                                                                          |

**Example Request (cURL):**

```bash
curl --request GET \
     --url 'https://public-api.birdeye.so/defi/v3/token/meta-data/single?address=So11111111111111111111111111111111111111112' \
     --header 'X-API-KEY: e218eef66dd64c3c9eaffc048daecfd4' \
     --header 'accept: application/json' \
     --header 'x-chain: solana'
```

**Response:**

```json
{
  "data": {
    "address": "So11111111111111111111111111111111111111112",
    "name": "Wrapped SOL",
    "symbol": "SOL",
    "decimals": 9,
    "extensions": {
      "coingecko_id": "solana",
      "serum_v3_usdc": "9wFFyRfZBsuAha4YcuxcXLKwMxJR43S7fPfQLusDBzvT",
      "serum_v3_usdt": "HWHvQhFmJB3NUcu1aihKmrKegfVxBEHzwVX6yZCKEsi1",
      "website": "https://solana.com/",
      "telegram": null,
      "twitter": "https://twitter.com/solana",
      "description": "Wrapped Solana ",
      "discord": "https://discordapp.com/invite/pquxPsq",
      "medium": "https://medium.com/solana-labs"
    },
    "logo_uri": "https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/So11111111111111111111111111111111111111112/logo.png"
  },
  "success": true
}
```

---

## Token List

/defi/v3/token/list

Retrieves a list of DeFi tokens based on optional parameters.

**Parameters:**

| Name                  | Type    | Required | Default     | Description                                                                                                                                                                                                                                                                                          |
| :-------------------- | :------ | :------- | :---------- | :--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `sort_by`             | string  | Yes      | liquidity   | Sort by one of the following: `liquidity`, `market_cap`, `fdv`, `recent_listing_time`, `holder`, `volume_1h_usd`, `volume_4h_usd`, `volume_8h_usd`, `volume_24h_usd`, `volume_1h_change_percent`, `volume_4h_change_percent`, `volume_8h_change_percent`, `volume_24h_change_percent`, `price_change_1h_percent`, `price_change_4h_percent`, `price_change_8h_percent`, `price_change_24h_percent`, `trade_1h_count`, `trade_4h_count`, `trade_8h_count`, `trade_24h_count` |
| `sort_type`           | string  | Yes      | desc        | Sort direction: `desc` or `asc`                                                                                                                                                                                                                                                                     |
| `min_liquidity`       | number  | No       |             | Minimum liquidity                                                                                                                                                                                                                                                                                |
| `max_liquidity`       | number  | No       |             | Maximum liquidity                                                                                                                                                                                                                                                                                |
| `min_market_cap`      | number  | No       |             | Minimum market cap                                                                                                                                                                                                                                                                               |
| `max_market_cap`      | number  | No       |             | Maximum market cap                                                                                                                                                                                                                                                                               |
| `min_fdv`             | number  | No       |             | Minimum fully diluted valuation                                                                                                                                                                                                                                                                     |
| `max_fdv`             | number  | No       |             | Maximum fully diluted valuation                                                                                                                                                                                                                                                                     |
| `min_recent_listing_time` | integer | No       |             | Minimum recent listing time                                                                                                                                                                                                                                                                          |
| `max_recent_listing_time` | integer | No       |             | Maximum recent listing time                                                                                                                                                                                                                                                                          |
| `min_holder`          | integer | No       |             | Minimum number of holders                                                                                                                                                                                                                                                                            |
| `min_volume_1h_usd`   | number  | No       |             | Minimum 1-hour volume (USD)                                                                                                                                                                                                                                                                         |
| `min_volume_2h_usd`   | number  | No       |             | Minimum 2-hour volume (USD)                                                                                                                                                                                                                                                                         |
| `min_volume_4h_usd`   | number  | No       |             | Minimum 4-hour volume (USD)                                                                                                                                                                                                                                                                         |
| `min_volume_8h_usd`   | number  | No       |             | Minimum 8-hour volume (USD)                                                                                                                                                                                                                                                                         |
| `min_volume_24h_usd`  | number  | No       |             | Minimum 24-hour volume (USD)                                                                                                                                                                                                                                                                        |
| `min_volume_1h_change_percent` | number  | No       |             | Minimum 1-hour volume change (%)                                                                                                                                                                                                                                                                     |
| `min_volume_2h_change_percent` | number  | No       |             | Minimum 2-hour volume change (%)                                                                                                                                                                                                                                                                     |
| `min_volume_4h_change_percent` | number  | No       |             | Minimum 4-hour volume change (%)                                                                                                                                                                                                                                                                     |
| `min_volume_8h_change_percent` | number  | No       |             | Minimum 8-hour volume change (%)                                                                                                                                                                                                                                                                     |
| `min_volume_24h_change_percent` | number  | No       |             | Minimum 24-hour volume change (%)                                                                                                                                                                                                                                                                    |
| `min_price_change_1h_percent` | number  | No       |             | Minimum 1-hour price change (%)                                                                                                                                                                                                                                                                      |
| `min_price_change_2h_percent` | number  | No       |             | Minimum 2-hour price change (%)                                                                                                                                                                                                                                                                      |
| `min_price_change_4h_percent` | number  | No       |             | Minimum 4-hour price change (%)                                                                                                                                                                                                                                                                      |
| `min_price_change_8h_percent` | number  | No       |             | Minimum 8-hour price change (%)                                                                                                                                                                                                                                                                      |
| `min_price_change_24h_percent` | number  | No       |             | Minimum 24-hour price change (%)                                                                                                                                                                                                                                                                     |
| `min_trade_1h_count`  | integer | No       |             | Minimum 1-hour trade count                                                                                                                                                                                                                                                                           |
| `min_trade_2h_count`  | integer | No       |             | Minimum 2-hour trade count                                                                                                                                                                                                                                                                           |
| `min_trade_4h_count`  | integer | No       |             | Minimum 4-hour trade count                                                                                                                                                                                                                                                                           |
| `min_trade_8h_count`  | integer | No       |             | Minimum 8-hour trade count                                                                                                                                                                                                                                                                           |
| `min_trade_24h_count` | integer | No       |             | Minimum 24-hour trade count                                                                                                                                                                                                                                                                          |
| `offset`              | integer | No       | 0           | Offset (0 to 10000)                                                                                                                                                                                                                                                                                 |
| `limit`               | integer | No       | 100         | Limit (1 to 100)                                                                                                                                                                                                                                                                                    |
| `x-chain`             | string  | No       | solana      | Chain: `solana`, `ethereum`, `arbitrum`, `avalanche`, `bsc`, `optimism`, `polygon`, `base`, `zksyncsui`                                                                                                                                                                                               |

**Example Request (cURL):**

```bash
curl --request GET \\
     --url 'https://public-api.birdeye.so/defi/v3/token/list?sort_by=liquidity&sort_type=desc&offset=0&limit=100' \\
     --header 'accept: application/json' \\
     --header 'x-chain: solana'
```

**Response:**

```json
{
  "data": {
    "items": [
      {
        "address": "So11111111111111111111111111111111111111112",
        "logo_uri": "https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/So11111111111111111111111111111111111111112/logo.png",
        "name": "Wrapped SOL",
        "symbol": "SOL",
        "decimals": 9,
        "extensions": {
          "coingecko_id": "solana",
          "serum_v3_usdc": "9wFFyRfZBsuAha4YcuxcXLKwMxJR43S7fPfQLusDBzvT",
          "serum_v3_usdt": "HWHvQhFmJB3NUcu1aihKmrKegfVxBEHzwVX6yZCKEsi1",
          "website": "https://solana.com/",
          "telegram": null,
          "twitter": "https://twitter.com/solana",
          "description": "Wrapped Solana ",
          "discord": "https://discordapp.com/invite/pquxPsq",
          "medium": "https://medium.com/solana-labs"
        },
        "market_cap": 82738000638.03874,
        "fdv": 100673483040.4702,
        "liquidity": 21156151741.04602,
        "last_trade_unix_time": 1740168344,
        "volume_1h_usd": 178555569.066242,
        "volume_1h_change_percent": 5.137282833843844,
        "volume_2h_usd": 339284991.8040391,
        "volume_2h_change_percent": -26.61569100721497,
        "volume_4h_usd": 832658170.8790178,
        "volume_4h_change_percent": -13.58784624531604,
        "volume_8h_usd": 1643878273.8022907,
        "volume_8h_change_percent": 46.434569826471375,
        "volume_24h_usd": 3990203112.7695704,
        "volume_24h_change_percent": 7.369520547148902,
        "trade_1h_count": 1326623,
        "trade_2h_count": 2538510,
        "trade_4h_count": 5268008,
        "trade_8h_count": 9491788,
        "trade_24h_count": 26794324,
        "price": 169.33280542782927,
        "price_change_1h_percent": -0.9997846994593949,
        "price_change_2h_percent": -1.2876469662583054,
        "price_change_4h_percent": -2.085219217559903,
        "price_change_8h_percent": -4.41500426252753,
        "price_change_24h_percent": -3.2849412034019183,
        "holder": 1306527,
        "recent_listing_time": null
      }
    ]
  },
  "success": true
}
```

---
