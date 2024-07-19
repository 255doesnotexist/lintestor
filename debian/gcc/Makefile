# Makefile for running tests and generating report.toml

PACKAGE_VERSION := 1.0.0
OS_VERSION := $(shell uname -r)

# Temporary directory for tests
TEMP_DIR := ./temp

# Test script
TEST_SCRIPT := ./run_tests.sh

# Report file
REPORT_FILE := report.toml

.PHONY: clean test check_dependencies verify_gcc generate_report install_gcc

# Clean up temporary files
clean:
	rm -rf $(TEMP_DIR) $(REPORT_FILE)

# Check for GCC and install if not present
check_dependencies: install_gcc

install_gcc:
	@echo "Checking for GCC and dependencies..."
	@if ! command -v gcc >/dev/null 2>&1; then \
		echo "GCC is not installed. Installing GCC..."; \
		sudo apt update && sudo apt install -y gcc; \
		if [ $$? -ne 0 ]; then \
			echo "Failed to install GCC. Please check your package manager settings."; \
			exit 1; \
		fi \
	else \
		echo "GCC is installed."; \
	fi

# Verify GCC by compiling a simple program
verify_gcc:
	@echo "Verifying GCC..."
	@mkdir -p $(TEMP_DIR)
	@echo 'int main() { return 0; }' > $(TEMP_DIR)/test.c
	@if ! gcc $(TEMP_DIR)/test.c -o $(TEMP_DIR)/test; then \
		echo "GCC compilation failed. Please check your GCC installation."; \
		exit 1; \
	fi
	@echo "GCC verification passed."

# Generate a report
generate_report:
	@echo "Generating report..."
	@echo "package_version = \"$(PACKAGE_VERSION)\"" > $(REPORT_FILE)
	@echo "os_version = \"$(OS_VERSION)\"" >> $(REPORT_FILE)
	@echo "[test_results]" >> $(REPORT_FILE)
	@$(TEST_SCRIPT) >> $(REPORT_FILE)

# Main test target
test: clean check_dependencies verify_gcc generate_report
	@echo "Tests completed. Report generated in $(REPORT_FILE)."
