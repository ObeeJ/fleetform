# Fleetform 🚀

Modern Infrastructure as Code tool built with **Rust + Go Fiber** that surpasses OpenTofu.

## ✨ Features

- **🔒 Memory-Safe**: Rust core with zero memory leaks  
- **⚡ High Performance**: Compiled binaries faster than Go runtime  
- **🌐 Modern UI**: Real-time web dashboard with WebSocket updates  
- **📊 Dependency Graphs**: Visual resource relationship mapping  
- **🔧 Module System**: Reusable configuration components  
- **🗄️ Multi-Backend**: File, S3, Consul state management  
- **🧪 Testing Framework**: Infrastructure validation  
- **🌍 Cross-Platform**: Windows, Linux, macOS support  

## 🚀 Quick Start

```bash
# Initialize workspace
cargo run -- init

# Create execution plan
cargo run -- plan

# Apply infrastructure
cargo run -- apply

# Start web dashboard
cd fiber && go run main.go
# Visit http://localhost:3001

📋 Commands

fleetform init                      # Initialize workspace
fleetform plan                      # Create execution plan
fleetform apply                     # Apply changes
fleetform providers                 # List providers
fleetform test                      # Run tests
fleetform workspace new <name>     # Create workspace

🌐 Web Dashboard

http://localhost:3001/ - Interactive dashboard

http://localhost:3001/ui - Plan data

http://localhost:3001/diff - Plan diff viewer

http://localhost:3001/modules - Module listing

ws://localhost:3001/realtime - Live WebSocket updates


🏗️ Architecture Overview

+-------------------+          +-----------------------+          +--------------------+
|                   |          |                       |          |                    |
|  CLI (Rust Core)   |  <---->  |   Go Fiber Web Server  |  <---->  |  Multi-Backend State|
|  - CLI commands    |          |  - Real-time Dashboard |          |    Management       |
|  - DAG & Planner   |          |  - WebSocket Updates   |          |  (File, S3, Consul) |
|  - Execution Plan  |          |                       |          |                    |
+-------------------+          +-----------------------+          +--------------------+

        |                               |                                  |
        |-------------------------------|----------------------------------|
                                        |
                            +-----------------------------+
                            |       Cloud & Infrastructure |
                            | - AWS for storage & config   |
                            | - Local/remote execution     |
                            +-----------------------------+

Rust Core:
Implements the core Infrastructure as Code logic — CLI, dependency graph (DAG), execution planning, and applying infrastructure changes safely with Rust’s memory guarantees.

Go Fiber Server:
Hosts a modern, real-time dashboard UI, powered by WebSocket for live updates and visualizing dependency graphs, plans, and diffs.

Multi-Backend State Management:
Supports multiple backend storages including local files, AWS S3 buckets, and Consul for flexible state persistence and distributed coordination.

Cloud & Infrastructure:
Fleetform interacts with cloud providers (AWS) for resource provisioning and uses environment configuration to manage credentials and regions.


🔧 Configuration

Create a .env file with the following variables:

AWS_ACCESS_KEY_ID=your_key
AWS_SECRET_ACCESS_KEY=your_secret
AWS_DEFAULT_REGION=us-east-1

🎯 Why Fleetform > OpenTofu

Feature	Fleetform	OpenTofu

Memory Safety	✅ Rust	❌ Go
Web Dashboard	✅ Real-time	❌ CLI only
Dependency Graphs	✅ Visual	❌ Text
Performance	✅ Compiled	❌ Runtime
Module Registry	✅ Built-in	❌ External


🤝 Contributing

1. Fork the repository


2. Create a feature branch


3. Make your changes


4. Run tests: cargo test


5. Submit a pull request
