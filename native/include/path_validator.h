// path_validator.h - Path validation utilities for secure file I/O
#pragma once

#include <filesystem>
#include <string>
#include <vector>
#include <optional>

namespace box_spread {
namespace security {

/**
 * PathValidator - Validates file paths to prevent directory traversal attacks
 * 
 * Security features:
 * - Validates paths against allowed base directories
 * - Resolves symlinks and canonical paths
 * - Prevents directory traversal sequences (../, ..\\)
 * - Ensures paths stay within allowed boundaries
 */
class PathValidator {
public:
  /**
   * Constructor
   * @param allowed_base_paths List of allowed base directories
   */
  explicit PathValidator(const std::vector<std::filesystem::path>& allowed_base_paths);

  /**
   * Validate a file path
   * @param file_path Path to validate (can be relative or absolute)
   * @param base_path Optional base path for relative paths (defaults to current directory)
   * @return Validated canonical path if valid, empty optional if invalid
   */
  std::optional<std::filesystem::path> validate_path(
      const std::filesystem::path& file_path,
      const std::filesystem::path& base_path = std::filesystem::current_path()) const;

  /**
   * Validate a file path (string overload)
   * @param file_path_str String path to validate
   * @param base_path Optional base path for relative paths
   * @return Validated canonical path if valid, empty optional if invalid
   */
  std::optional<std::filesystem::path> validate_path(
      const std::string& file_path_str,
      const std::filesystem::path& base_path = std::filesystem::current_path()) const;

  /**
   * Check if a path contains directory traversal sequences
   * @param path Path to check
   * @return true if path contains traversal sequences
   */
  static bool contains_traversal(const std::filesystem::path& path);

  /**
   * Get allowed base paths
   * @return Vector of allowed base paths
   */
  const std::vector<std::filesystem::path>& get_allowed_base_paths() const {
    return allowed_base_paths_;
  }

private:
  std::vector<std::filesystem::path> allowed_base_paths_;

  /**
   * Check if a canonical path is within any allowed base path
   * @param canonical_path Canonical path to check
   * @return true if path is within allowed boundaries
   */
  bool is_within_allowed_boundaries(const std::filesystem::path& canonical_path) const;
};

/**
 * Create a default path validator with common allowed directories
 * @param additional_paths Additional allowed paths (optional)
 * @return PathValidator instance
 */
PathValidator create_default_path_validator(
    const std::vector<std::filesystem::path>& additional_paths = {});

} // namespace security
} // namespace box_spread
