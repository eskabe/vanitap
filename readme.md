# Vanitap

A Bitcoin Taproot (P2TR) vanity address generator.  
Searches for user-defined prefix or suffix patterns in Taproot addresses.  
Optionally, restricts results to a matching payment (P2WPKH) address pattern.

> This project was inspired by [tapvanitygen](https://github.com/ordinals-wallet/tapvanitygen)  
> All code in this repository is written and maintained independently.

---

## Build
```bash
cargo build --release
```

---

## Usage

### Start vanitap
> ./target/release/vanitap

**Alternative:**   
You can also run the program directly via Cargo in release mode without manually invoking the binary:  
> cargo run --release -- [ARGS]  

### Command line arguments:

| Argument       | Short | Description                                                                  | Default        |
| -------------- | ----- | -----------------------------------------------------------------------------| -------------- |
| `--suffix`     | `-s`  | Search for patterns as **suffix** instead of prefix                          | `false`        |
| `--pattern`    | `-p`  | Single pattern to search for (instead of filename)                           | -              |
| `--filename`   | `-f`  | Filename to load patterns from                                               | `patterns.txt` |
| `--patternpay` | `-y`  | Single pattern for payment address (p2wpkh) instead of patterns_payment.txt  | -              |
| `--help`       | `-h`  | Print help                                                                   | -              |
| `--version`    | `-V`  | Print version                                                                | -              |

### Files:

* `patterns.txt` 
* `patterns_payment.txt`

**Format**
* One pattern per line
* Empty lines are ignored

**ONLY use bech32:**
* 023456789acdefghjklmnpqrstuvwxyz

**Note:** `patterns_payment.txt` will not be updated on a hit; all entries are used for every search.  
-> Keep the file empty to search without payment patterns.

---

## Output

**Console output includes:**

* Pattern found
* Taproot address (P2TR)
* Optional payment address (P2WPKH)
* Private key
* Public key
* XOnlyPublicKey
* ScriptPubKey
* Performance info in H/s, kH/s, MH/s, GH/s

**Results file (`results.txt`) format:**

```
Pattern: <pattern>
p2tr: <taproot_address>
p2wpkh: <payment_address>
PrivateKey: <secret>
PublicKey: <public>
XOnlyPublicKey: <xonly>
ScriptPubKey: <script_pubkey>
```

**Pattern file (`patterns.txt`)** is updated automatically as patterns are matched.

---

## Notes & Behavior

* Payment pattern file (`patterns_payment.txt`) is **static**; all entries are applied to every Taproot pattern.  
Keep the file empty to search without payment pattern.
* Script continues until **all Taproot patterns are matched**.
* Uses multi-threaded generation via **rayon** for high performance.
* Supports both **prefix** and **suffix** searches.
* All patterns are optional; if none are provided, no addresses are generated.  
* Use Ctrl + C to terminate the generator at any time.

---

## Examples

**1. Prefix search from file `patterns.txt`**

```bash
 ./target/release/vanitap
```

**2. Suffix search from file `patterns.txt`**

```bash
 ./target/release/vanitap -s
```

**3. Single pattern from CLI**

```bash
 ./target/release/vanitap -p w4ll3t
```

**4. Single pattern from CLI with payment pattern from CLI**

```bash
 ./target/release/vanitap -p w4ll3t -y pay
```

**5. Using custom patterns file**

```bash
 ./target/release/vanitap -f my_patterns_file.txt
```

---

### Full example with suffix and payment pattern

Command:
```bash
 ./target/release/vanitap -p w4ll3t -s -y pay
```
Alternative:  
```bash
cargo run --release -- -p w4ll3t -s -y pay
```

*Output:*

```
Pattern from console input.
Pay-pattern from console input.
Search for suffix(es):
w4ll3t
...with payment (p2wpkh) suffix(es):
pay
252.29 kH/s
254.03 kH/s
253.30 kH/s
```