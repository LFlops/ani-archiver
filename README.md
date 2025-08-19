# Ani Archiver

[![codecov](https://codecov.io/gh/LFlops/ani-archiver/branch/develop/graph/badge.svg)](https://codecov.io/gh/LFlops/ani-archiver)


A command-line tool to scrape TV show information from The Movie Database (TMDB), organize video files, and create NFO files for media centers like Kodi.

## Features

*   Scrapes TV show details from TMDB.
*   Organizes video files into a structured directory format.
*   Creates `tvshow.nfo` files compatible with media centers.
*   Caches TMDB IDs and file hashes to avoid redundant lookups.
*   Supports both season/episode and single file naming conventions.
## architecture 
```mermaid
%% 这是一个流程图 (Top-Down 方向)
%% 这是一个流程图 (Top-Down 方向)
flowchart TD
    subgraph "准备阶段"
        A[用户 User] -->|编辑| B(配置文件 config.yaml);
    end

    subgraph "执行阶段"
    A -->|运行| C{ani_archiver};
    B -->|读取配置| C;
    C -->|1. 扫描| D[源动画文件 <br> 本地硬盘];
    D -->|2. 提取文件夹名| C;
    C -->|3. 查询信息| E[(TMDB API)];
    E -->|4. 返回元数据| C;
    C -->|5. 根据元数据和配置重命名/移动| F[整理后的目录 <br> 本地硬盘];
    end

subgraph "最终结果"
F --> G((完成));
end

%%%% 样式定义
%%style A fill:#f9f,stroke:#333,stroke-width:2px
%%style C fill:#bbf,stroke:#333,stroke-width:2px
%%style E fill:#9f9,stroke:#333,stroke-width:2px
```

```mermaid
%% 这是一个序列图
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
## Usage

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/your-username/ani-archiver.git
    cd ani-archiver
    ```

2.  **Create a `.env` file:**
    ```
    TMDB_API_KEY=your_tmdb_api_key
    ```

3.  **Run the archiver:**
    ```bash
    cargo run -- --source /path/to/your/shows --dest /path/to/organized/shows
    ```
4. **result **
* when program run successfully, it will output a message like this:
```shell
    - .nfo file created for show
    - .processed.json file created for marker the processed
```
## Testing and Coverage

This project uses `cargo test` for running unit and integration tests.

### Running Tests

To run the tests, use the following command:

```bash
cargo test
```

### Test Coverage

To generate a test coverage report, you can use `grcov`.

1.  **Install `grcov` and the necessary LLVM tools:**
    ```bash
    cargo install grcov
    rustup component add llvm-tools-preview
    ```

2.  **Run the tests with coverage enabled:**
    ```bash
    CARGO_INCREMENTAL=0 RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort" RUSTDOCFLAGS="-Cpanic=abort" cargo test
    ```

3.  **Generate the coverage report:**
    ```bash
    grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing -o ./target/debug/coverage/
    ```

This will generate an HTML report in the `target/debug/coverage/` directory.