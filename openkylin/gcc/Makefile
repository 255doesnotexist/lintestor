PACKAGE_NAME := gcc
PACKAGE_VERSION := $(shell apt-cache policy $(PACKAGE_NAME) | grep Installed | awk '{print $$2}')
OS_VERSION := $(shell uname -v)
KERNEL_VERSION := $(shell uname -r)

# Temporary directory for tests
TEMP_DIR := ./temp

# Test script
GENERATE_REPORT_SCRIPT := ./run_tests.sh

.PHONY: clean test check_dependencies verify_gcc generate_report install_gcc

# Clean up temporary files
clean:
	@rm -rf $(TEMP_DIR)

# Check for GCC and install if not present
check_dependencies: install_gcc

install_gcc:
	@echo "Checking for GCC and dependencies..."
	@command -v gcc >/dev/null 2>&1 || { \
		echo "GCC is not installed. Installing GCC..."; \
		sudo apt update && sudo apt install -y gcc; \
		if [ $$? -ne 0 ]; then \
			echo "Failed to install GCC. Please check your package manager settings."; \
			exit 1; \
		fi \
	} && echo "GCC is installed."

# Verify GCC by compiling a simple program
verify_gcc:
	@echo "Verifying GCC..."
	@mkdir -p $(TEMP_DIR)
	@echo 'int main() { return 0; }' > $(TEMP_DIR)/test.c
	@! gcc $(TEMP_DIR)/test.c -o $(TEMP_DIR)/test && { \
		echo "GCC compilation failed. Please check your GCC installation."; \
		exit 1; \
	} || echo "GCC verification passed."

# Generate a report
generate_report:
	@$(GENERATE_REPORT_SCRIPT) $(TEMP_DIR)/test

# Main test target
test: clean check_dependencies verify_gcc generate_report
	@echo "Tests completed."