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

**🎉 ALL PHASES COMPLETE - PRODUCTION READY!**

**Phase 1: Core Harness** ✅
- [x] Challenge definition structures
- [x] Basic project architecture
- [x] Sample challenge format
- [x] Rust foundation with proper error handling
- [x] GitHub repository with continuous integration

**Phase 2: Implementation** ✅
- [x] OpenRouter LLM integration
- [x] Docker-based code execution
- [x] Challenge-solving orchestrator
- [x] Metrics collection and analysis

**Phase 3: Self-Improvement** ✅
- [x] SWE-Forge integration for challenge generation
- [x] Performance analysis and optimization
- [x] Automated improvement deployment
- [x] Continuous feedback loops

## 🎮 Usage

### Basic Usage

```bash
# Run the main harness demonstration
cargo run

# Run the SWE-Forge integration demo
cargo run --bin demo-swe-forge

# Build the project
cargo build --release
```

### Environment Setup

```bash
# Set your OpenRouter API key
export OPENROUTER_API_KEY="your-api-key-here"

# Set GitHub token for SWE-Forge integration
export GITHUB_TOKEN="your-github-token-here"
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

## 🎯 Production Capabilities

The Ninja harness now provides complete end-to-end functionality:

### 🔄 Continuous Challenge Generation
- **SWE-Forge Integration**: Mines real GitHub PRs for synthetic challenge generation
- **Language Support**: Python, Rust, JavaScript and extensible architecture
- **Difficulty Classification**: Automatic complexity scoring and categorization
- **Real-world Problems**: Generated from actual software engineering challenges

### 🧠 AI-Powered Problem Solving
- **Claude 3.5 Sonnet**: State-of-the-art LLM for code generation
- **OpenRouter Integration**: Scalable API access with cost optimization
- **Context-Aware Generation**: Understands challenge requirements and constraints
- **Multi-language Support**: Handles various programming languages

### 🐳 Safe Code Execution
- **Docker Sandboxing**: Isolated container execution for security
- **Test Automation**: Automatic pytest and test framework integration
- **Performance Monitoring**: Execution time and resource usage tracking
- **Error Capture**: Comprehensive logging of failures and diagnostics

### 📊 Performance Analytics
- **Success Rate Tracking**: Monitor solving accuracy over time
- **Cost Analysis**: Token usage and API cost optimization
- **Trend Detection**: Identify performance improvements and regressions
- **Automated Recommendations**: AI-powered suggestions for optimization

### 🚀 Self-Improvement Loop
- **Continuous Learning**: Analyzes failures and improves strategies
- **Automated Optimization**: Updates solving approaches based on performance data
- **Challenge Evolution**: Generates increasingly complex challenges
- **System Evolution**: Self-modifying codebase for enhanced capabilities

## 🧪 Demo Commands

```bash
# Core functionality demonstration
cargo run

# Full SWE-Forge integration pipeline
cargo run --bin demo-swe-forge

# Build optimized production version
cargo build --release
```

---

**Current Version**: 1.0.0 (Production Ready)
**Status**: All three phases complete - fully operational self-improving system
**Repository**: https://github.com/unconst/ninja-coding-harness