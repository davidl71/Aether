// path_validator.cpp - Path validation implementation
#include "path_validator.h"
#include <spdlog/spdlog.h>
#include <algorithm>
#include <cctype>

namespace box_spread {
namespace security {

PathValidator::PathValidator(const std::vector<std::filesystem::path>& allowed_base_paths)
  : allowed_base_paths_(allowed_base_paths) {
  // Normalize all allowed base paths to canonical form
  for (auto& path : allowed_base_paths_) {
    try {
      if (std::filesystem::exists(path)) {
        path = std::filesystem::canonical(path);
      } else {
        // Path doesn't exist yet, but we'll allow it (parent directory might exist)
        path = std::filesystem::absolute(path);
        path = path.lexically_normal();
      }
    } catch (const std::filesystem::filesystem_error& e) {
      spdlog::warn("Failed to normalize allowed base path {}: {}", path.string(), e.what());
    }
  }
}

std::optional<std::filesystem::path> PathValidator::validate_path(
    const std::filesystem::path& file_path,
    const std::filesystem::path& base_path) const {
  try {
    // Check for directory traversal sequences
    if (contains_traversal(file_path)) {
      spdlog::warn("Path validation failed: contains directory traversal sequences: {}",
                   file_path.string());
      return std::nullopt;
    }

    // Resolve path (handle relative paths)
    std::filesystem::path resolved_path;
    if (file_path.is_absolute()) {
      resolved_path = file_path;
    } else {
      resolved_path = base_path / file_path;
    }

    // Normalize path (remove redundant separators, resolve . and ..)
    resolved_path = resolved_path.lexically_normal();

    // Get canonical path (resolve symlinks, absolute path)
    std::filesystem::path canonical_path;
    if (std::filesystem::exists(resolved_path)) {
      canonical_path = std::filesystem::canonical(resolved_path);
    } else {
      // File doesn't exist yet (e.g., for writing), but we can still validate the parent
      canonical_path = std::filesystem::absolute(resolved_path);
      canonical_path = canonical_path.lexically_normal();
      
      // Check parent directory instead
      if (canonical_path.has_parent_path()) {
        auto parent = canonical_path.parent_path();
        if (std::filesystem::exists(parent)) {
          canonical_path = std::filesystem::canonical(parent) / canonical_path.filename();
        }
      }
    }

    // Check if path is within allowed boundaries
    if (!is_within_allowed_boundaries(canonical_path)) {
      spdlog::warn("Path validation failed: path outside allowed boundaries: {}",
                   canonical_path.string());
      return std::nullopt;
    }

    return canonical_path;
  } catch (const std::filesystem::filesystem_error& e) {
    spdlog::error("Path validation error: {}: {}", file_path.string(), e.what());
    return std::nullopt;
  } catch (const std::exception& e) {
    spdlog::error("Unexpected error during path validation: {}: {}", file_path.string(), e.what());
    return std::nullopt;
  }
}

std::optional<std::filesystem::path> PathValidator::validate_path(
    const std::string& file_path_str,
    const std::filesystem::path& base_path) const {
  return validate_path(std::filesystem::path(file_path_str), base_path);
}

bool PathValidator::contains_traversal(const std::filesystem::path& path) {
  std::string path_str = path.string();
  
  // Check for common directory traversal patterns
  // Look for ".." sequences (case-insensitive on Windows)
  std::string lower_path = path_str;
  std::transform(lower_path.begin(), lower_path.end(), lower_path.begin(),
                 [](unsigned char c) { return std::tolower(c); });
  
  // Check for "../" or "..\\" patterns
  if (lower_path.find("../") != std::string::npos ||
      lower_path.find("..\\") != std::string::npos) {
    return true;
  }
  
  // Check for ".." at start or after separators
  if (lower_path.find("/../") != std::string::npos ||
      lower_path.find("\\..\\") != std::string::npos ||
      lower_path.find("/..") == lower_path.length() - 3 ||
      lower_path.find("\\..") == lower_path.length() - 3) {
    return true;
  }
  
  // Check for encoded traversal sequences (%2e%2e%2f, %2e%2e%5c)
  if (lower_path.find("%2e%2e%2f") != std::string::npos ||
      lower_path.find("%2e%2e%5c") != std::string::npos) {
    return true;
  }
  
  return false;
}

bool PathValidator::is_within_allowed_boundaries(
    const std::filesystem::path& canonical_path) const {
  // If no allowed paths specified, reject all (fail-safe)
  if (allowed_base_paths_.empty()) {
    return false;
  }
  
  // Check if canonical path is within any allowed base path
  for (const auto& allowed_base : allowed_base_paths_) {
    try {
      // Check if canonical_path starts with allowed_base
      auto [mismatch, _] = std::mismatch(
          canonical_path.begin(), canonical_path.end(),
          allowed_base.begin(), allowed_base.end());
      
      // If we've consumed all of allowed_base, then canonical_path is within it
      if (mismatch == allowed_base.end()) {
        return true;
      }
    } catch (const std::exception& e) {
      spdlog::warn("Error checking path boundaries: {}", e.what());
      continue;
    }
  }
  
  return false;
}

PathValidator create_default_path_validator(
    const std::vector<std::filesystem::path>& additional_paths) {
  std::vector<std::filesystem::path> allowed_paths;
  
  // Add current working directory
  try {
    allowed_paths.push_back(std::filesystem::current_path());
  } catch (const std::filesystem::filesystem_error& e) {
    spdlog::warn("Failed to get current directory: {}", e.what());
  }
  
  // Add user home directory (if available)
  const char* home = std::getenv("HOME");
  if (home) {
    try {
      allowed_paths.push_back(std::filesystem::path(home));
    } catch (const std::exception& e) {
      spdlog::warn("Failed to add HOME directory: {}", e.what());
    }
  }
  
  // Add user config directory
  if (home) {
    try {
      auto config_dir = std::filesystem::path(home) / ".config" / "ib_box_spread";
      allowed_paths.push_back(config_dir);
    } catch (const std::exception& e) {
      spdlog::warn("Failed to add config directory: {}", e.what());
    }
  }
  
  // Add any additional paths
  allowed_paths.insert(allowed_paths.end(), additional_paths.begin(), additional_paths.end());
  
  return PathValidator(allowed_paths);
}

} // namespace security
} // namespace box_spread
