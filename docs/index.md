# Ani Archiver

[![codecov](https://codecov.io/gh/LFlops/ani-archiver/branch/develop/graph/badge.svg)](https://codecov.io/gh/LFlops/ani-archiver)
[![License](https://img.shields.io/github/license/LFlops/ani-archiver)](https://github.com/LFlops/ani-archiver)
[![Rust](https://img.shields.io/badge/built_with-Rust-dca282.svg)](https://www.rust-lang.org/)

**Ani Archiver** is a command-line tool designed to scrape TV show information from The Movie Database (TMDB), organize video files, and create NFO files for media centers like Kodi.

---

## ✨ Features

* **Scrapes TV show details** from TMDB.
* **Organizes video files** into a structured directory format.
* **Creates `tvshow.nfo` files** compatible with media centers.
* **Caches TMDB IDs** and file hashes to avoid redundant lookups.
* **Supports multiple naming conventions** (both season/episode and single file).

---

## 🛠 Architecture

### Workflow Overview

```mermaid
flowchart TD
    subgraph "Preparation"
        A[User] -->|Edit| B(配置文件 config.yaml);
    end

    subgraph "Execution"
        A -->|Run| C{ani_archiver};
        B -->|Read Config| C;
        C -->|1. Scan| D[源动画文件 <br> 本地硬盘];
        D -->|2. Extract Folder Names| C;
        C -->|3. Query Info| E[(TMDB API)];
        E -->|4. Return Metadata| C;
        C -->|5. Rename & Move| F[整理后的目录 <br> 本地硬盘];
    end

    subgraph "Result"
        F --> G((Done));
    end
```

### Process Sequence

```mermaid
sequenceDiagram
    participant U as 用户
    participant A as ani_archiver
    participant C as 配置文件
    participant H1 as 源硬盘
    participant TMDB
    participant H2 as 目标硬盘

    U->>+A: 运行脚本
    A->>C: 读取配置()
    C-->>A: 返回配置参数

    A->>H1: 扫描目录()
    H1-->>A: 返回文件列表

    loop 遍历每个文件
        A->>TMDB: 查询动画信息(文件名)
        activate TMDB
        TMDB-->>A: 返回元数据 (成功/失败)
        deactivate TMDB

        alt 信息查询成功
            A->>H2: 创建目录/重命名/移动文件()
        else 信息查询失败
            A->>A: 记录日志/跳过文件
        end
    end

    A-->>-U: 执行完毕
```

---

## 🚀 Usage

### 1. Installation

Clone the repository:

```bash
git clone [https://github.com/your-username/ani-archiver.git](https://github.com/your-username/ani-archiver.git)
cd ani-archiver
```

### 2. Configuration

Create a `.env` file in the project root:

```properties
TMDB_API_KEY=your_tmdb_api_key
```

### 3. Run the Archiver

Execute the program using Cargo:

```bash
cargo run -- --source /path/to/your/shows --dest /path/to/organized/shows
```

!!! success "Expected Output"
When the program runs successfully, it will output messages indicating completion:

    * ✅ `.nfo` local_file created for show
    * ✅ `.processed.json` local_file created for marker the processed

---

## 🧪 Testing and Coverage

This project uses `cargo test` for running unit and integration tests.

### Running Tests

To run the tests, use the following command:

```bash
cargo test
```

### Test Coverage

To generate a test coverage report, you can use `grcov`.

**1. Install Dependencies:**

```bash
cargo install grcov
rustup component add llvm-tools-preview
```

**2. Run Tests with Coverage Enabled:**

```bash
CARGO_INCREMENTAL=0 RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort" RUSTDOCFLAGS="-Cpanic=abort" cargo test
```

**3. Generate HTML Report:**

```bash
grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing -o ./target/debug/coverage/
```

!!! tip "View Report"
Open `target/debug/coverage/index.html` in your browser to view the detailed coverage report.