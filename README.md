# Mimi

Mimi is an AI agent harness written in Rust. It uses OpenRouter's Responses API to work with your codebase through file search, file editing, and shell tools.

## Features

- Interactive terminal chat
- JSONL mode for programmatic integrations
- Indexed file and content search
- Read, write, edit, and Bash tools with approval prompts for mutating actions
- Session logs and task todo files stored locally
- Automatic context compaction for longer conversations

## Built-in tools

| Tool        | Purpose                                                  | Approval required |
| ----------- | -------------------------------------------------------- | ----------------- |
| File search | Finds files and content in the current working directory | No                |
| Read file   | Reads a file                                             | No                |
| Write file  | Creates or replaces a file                               | Yes               |
| Edit file   | Applies targeted text edits                              | Yes               |
| Bash        | Runs a shell command                                     | Yes               |
| Todo list   | Creates a session task list                              | No                |

## Requirements

- An [OpenRouter](https://openrouter.ai/) API key

## Setup

Clone the repository and set your API key:

```bash
git clone https://github.com/yogesharc/mimi.git
cd mimi
export OPENROUTER_API_KEY="your-api-key"
```

You can also place the key in a `.env` file in the project directory:

```dotenv
OPENROUTER_API_KEY=your-api-key
```

## Usage

Run Mimi in interactive mode:

```bash
cargo run
```

Ask it to inspect or change the current project. Type `exit` to end the session. Mimi requests confirmation before it writes or edits files, or runs Bash commands.

Choose a model and reasoning effort when needed:

```bash
cargo run -- --model openai/gpt-5.4-mini --effort medium
```

Available effort levels are `minimal`, `low`, `medium`, and `high`. The default model is `openai/gpt-5.6-sol`.

### JSONL mode

Use `json` mode to exchange newline-delimited JSON with Mimi:

```bash
cargo run -- json
```

Send a prompt:

```json
{ "type": "prompt", "text": "Explain the project structure." }
```

When Mimi needs permission for a write, edit, or Bash call, respond with the matching call ID:

```json
{ "type": "approval", "call_id": "call_123", "approved": true }
```

End the session with:

```json
{ "type": "exit" }
```

## Development

Run the test suite:

```bash
cargo test
```

Format the code before contributing:

```bash
cargo fmt
```

## License

MIT.
