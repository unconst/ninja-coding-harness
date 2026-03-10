# 🥷 Ninja Coding Harness

A self-improving coding challenge solver built in Rust, designed to continuously enhance its problem-solving capabilities through synthetic challenge generation and feedback loops.

## 🎯 Overview

Ninja is a sophisticated coding harness that:

- **Solves Programming Challenges**: Uses LLM integration to solve SWE-bench style coding challenges
- **Self-Improves**: Analyzes performance and automatically optimizes problem-solving strategies
- **Generates Challenges**: Creates infinite sequences of synthetic coding challenges from real GitHub PRs
- **Tracks Metrics**: Comprehensive performance analysis and cost optimization

## 🏗️ Architecture

The system is built around several core components:

- **Challenge Management**: JSON-based challenge definitions with comprehensive test suites
- **LLM Integration**: OpenRouter API integration for code generation and analysis
- **Docker Sandbox**: Isolated execution environment for safe code testing
- **Improvement Loop**: Continuous analysis and optimization based on performance metrics
- **SWE-Forge Pipeline**: Synthetic challenge generation from real GitHub pull requests

## 🚀 Current Status

**Phase 1: Core Harness** ✅
- [x] Challenge definition structures
- [x] Basic project architecture
- [x] Sample challenge format
- [x] Rust foundation with proper error handling

**Phase 2: Implementation** 🚧
- [ ] OpenRouter LLM integration
- [ ] Docker-based code execution
- [ ] Challenge-solving orchestrator
- [ ] Metrics collection and analysis

**Phase 3: Self-Improvement** 📋
- [ ] SWE-Forge integration for challenge generation
- [ ] Performance analysis and optimization
- [ ] Automated improvement deployment
- [ ] Continuous feedback loops

## 🎮 Usage

### Basic Usage

```bash
# Run the harness (currently shows demo output)
cargo run

# Build the project
cargo build --release
```

### Environment Setup

```bash
# Set your OpenRouter API key
export OPENROUTER_API_KEY="your-api-key-here"
```

## 📄 Challenge Format

Challenges are defined in JSON format with comprehensive test suites:

```json
{
  "id": "unique-challenge-id",
  "title": "Challenge Title",
  "description": "Detailed problem description",
  "language": "python",
  "difficulty": "Easy",
  "tests": [
    {
      "name": "test_basic_functionality",
      "test_type": "FailToPass",
      "command": "python -m pytest test_file.py::test_basic -v"
    }
  ],
  "setup_commands": ["pip install pytest"],
  "expected_files": ["solution.py", "test_file.py"]
}
```

## 🔧 Technical Details

### Dependencies

- **Rust Edition 2021**: Modern Rust features and performance
- **Serde**: JSON serialization for challenge definitions
- **OpenRouter API**: LLM integration for code generation
- **Docker**: Sandboxed execution environment
- **Tokio**: Async runtime for concurrent operations

### Architecture Patterns

Based on research from [SWE-Forge](https://github.com/CortexLM/swe-forge), the harness implements:

- **Async-first design** with proper concurrency management
- **Typed error handling** throughout the system
- **Docker sandboxing** for secure code execution
- **Structured LLM output** using function calling
- **Real-time metrics** collection and analysis

## 🎯 Roadmap

### Immediate Next Steps
1. Implement LLM integration with OpenRouter
2. Add Docker-based code execution
3. Create challenge-solving orchestrator
4. Integrate metrics collection

### Medium Term
1. SWE-Forge integration for synthetic challenges
2. Performance analysis and optimization
3. Automated improvement loops
4. Cost optimization strategies

### Long Term
1. Multi-language challenge support
2. Advanced difficulty classification
3. Distributed execution capabilities
4. Community challenge sharing

## 🤝 Contributing

This is currently a prototype implementation. The architecture is designed to be:

- **Modular**: Easy to extend with new components
- **Type-safe**: Leveraging Rust's strong type system
- **Performance-focused**: Async operations and efficient resource usage
- **Research-driven**: Based on proven SWE-bench methodologies

## 📊 Performance Goals

Target metrics for the completed system:

- **Challenge Solve Rate**: >80% on SWE-bench style problems
- **Processing Speed**: <5 minutes average per challenge
- **Cost Efficiency**: <$0.10 per solved challenge
- **Accuracy**: >95% test pass rate on solved challenges

## 📝 License

This project is part of the Arbos coding agent ecosystem and follows open development principles.

---

**Current Version**: 0.1.0 (Prototype)
**Status**: Foundation implemented, LLM integration in progress
**Next Milestone**: Working challenge solver with Docker execution