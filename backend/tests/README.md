# How to Use Backend Tests

1.  **Ensure `cargo-watch` is installed:**
    ```bash
    cargo install cargo-watch --locked
    ```
2.  **Run the application server (Terminal 1):**
    Watches the `src/` directory and restarts the server on changes.
    ```bash
    cargo watch -q -c -w src/ -x run
    ```
3.  **Run a specific test file (Terminal 2):**
    Watches the `tests/` directory and re-runs the specified test file on changes.
    Replace `test_testname` with the actual test file name (e.g., `test_users`, `test_projects`).
    ```bash
    cargo watch -q -c -w tests/ -x "test -q test_testname -- --nocapture"
    ```
    *	The `-- --nocapture` flag ensures you see `println!` output from tests.*

You should now see live updates of API calls and test results as you modify the code.