# Word Validator API

A fast, memory-efficient word validation API built in Rust. This API validates words against a local dictionary and falls back to external APIs when needed.

## Features

- **Memory Efficient**: Binary size under 3.2MB (well below 25MB requirement)
- **Fast Performance**: Uses HashSet for O(1) lookup time
- **Local Dictionary**: Reads from `dictionary.txt` file, strips symbols, keeps only alphanumeric words
- **External API Fallback**: Queries 3 free APIs when word is not found locally:
  - Free Dictionary API (dictionaryapi.dev)
  - Datamuse API
  - Wordnik API (with test key for demonstration)
- **REST API**: Simple HTTP endpoint for word validation

## Prerequisites

- Rust 1.70 or higher
- Cargo (comes with Rust)

## Installation

1. Clone the repository:
```bash
git clone https://github.com/AstroX11/word-vaildator-api.git
cd word-vaildator-api
```

2. Build the project:
```bash
cargo build --release
```

The optimized binary will be available at `target/release/word_validator_api`

## Usage

### Starting the Server

```bash
cargo run --release
```

Or run the binary directly:
```bash
./target/release/word_validator_api
```

The server will start on `http://0.0.0.0:8080`

### API Endpoints

#### GET /

Returns API information and statistics.

**Example:**
```bash
curl http://localhost:8080/
```

**Response:**
```json
{
  "service": "Word Validator API",
  "version": "0.1.0",
  "usage": "/word?word=<word_to_validate>",
  "dictionary_size": 120
}
```

#### GET /word?word={word}

Validates if a word exists in the dictionary.

**Parameters:**
- `word` (required): The word to validate

**Example:**
```bash
curl "http://localhost:8080/word?word=books"
```

**Response:**
```json
{
  "word": "books",
  "found": true,
  "source": "local"
}
```

**Response Fields:**
- `word`: The queried word (lowercase)
- `found`: Boolean indicating if the word was found
- `source`: Where the word was found (`local`, `external`, or `none`)

### Example Queries

```bash
# Word found in local dictionary
curl "http://localhost:8080/word?word=hello"
# {"word":"hello","found":true,"source":"local"}

# Word not found locally, checking external APIs
curl "http://localhost:8080/word?word=elephant"
# {"word":"elephant","found":true,"source":"external"}

# Invalid word not found anywhere
curl "http://localhost:8080/word?word=invalidword123"
# {"word":"invalidword123","found":false,"source":"none"}
```

## Dictionary File

The `dictionary.txt` file contains words separated by spaces. The API:
- Splits text by whitespace
- Strips all non-alphanumeric characters from each word
- Converts all words to lowercase
- Filters out empty strings
- Loads words into memory once at startup

**Example dictionary.txt format:**
```
hello world test example book books reading
@#$%^ invalid!@# symbols###here test123
computer science programming rust
```

Words with symbols like `test123` will be stripped to `test` and `symbols###here` becomes `symbolshere`.

## Configuration

### Port Configuration

To change the port, modify the bind address in `src/main.rs`:
```rust
.bind(("0.0.0.0", 8080))?  // Change 8080 to your desired port
```

### External API Timeout

API calls timeout after 5 seconds. To modify, change in `src/main.rs`:
```rust
.timeout(std::time::Duration::from_secs(5))  // Change 5 to desired seconds
```

## Development

### Running in Development Mode

```bash
cargo run
```

### Running Tests

```bash
cargo test
```

### Building for Production

```bash
cargo build --release
```

The release build applies optimizations for minimal binary size:
- `opt-level = "z"` - Optimize for size
- `lto = true` - Link-time optimization
- `codegen-units = 1` - Single codegen unit
- `strip = true` - Strip symbols

## Performance

- **Binary Size**: ~3.2MB (compressed and stripped)
- **Memory Usage**: Minimal - dictionary loaded once at startup
- **Lookup Time**: O(1) for local dictionary (HashSet)
- **External API Timeout**: 5 seconds per API

## API Response Times

- Local dictionary: < 1ms
- External APIs: 100-500ms (depends on network and API availability)

## Error Handling

- Missing `word` parameter: Returns 400 Bad Request
- External API failures: Silently fall back to next API
- Dictionary file missing: Starts with empty dictionary

## Dependencies

- `actix-web`: Web framework
- `tokio`: Async runtime
- `serde`: Serialization
- `reqwest`: HTTP client for external APIs
- `once_cell`: Lazy static initialization

## License

See LICENSE file for details.

## Contributing

1. Fork the repository
2. Create your feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request

## Support

For issues and questions, please open an issue on GitHub.
