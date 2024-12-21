# shellcheck disable=SC2148
shopt -s globstar nullglob

extract_package_info() {
  local cargo_file="$1"
  local package_name
  local package_version
  package_name=$(toml get "$cargo_file" package.name | tr -d '"')
  package_version=$(toml get "$cargo_file" package.version | tr -d '"')
  # Only output if both package name and version are present
  if [[ -n "$package_name" && -n "$package_version" ]]; then
    echo "$package_name" "$package_version"
  fi
}

try_publishing_crate() {
  while IFS= read -r -d '' cargo_file; do
    # Try to extract the package name and version for the current Cargo.toml file
    read -r package_name package_version < <(extract_package_info "$cargo_file")
    echo "Found Cargo.toml file: $(realpath "$cargo_file")"

    # Check if the package name or version is empty and skip if it is
    # This is for any top level Cargo.toml files that specify workspace members
    if [[ -z "$package_name" ]] || [[ -z "$package_version" ]]; then
      echo "Could not find a package name or version in this file."
      echo ""
      continue
    fi

    echo "Found package: $package_name"
    echo "Found version: $package_version"

    if ! cargo info "$package_name@$package_version"; then
      if [[ " $* " == *" dryrun "* ]]; then
        cargo publish \
          --package "$package_name" \
          --dry-run \
          --allow-dirty
      elif [[ " $* " == *" publish "* ]]; then
        CARGO_HOME=$(realpath .cargo/)
        cargo publish \
          --package "$package_name" \
          --token "$CARGO_REGISTRY_TOKEN"
      else
        echo "I don't know what you want me to do."
      fi
    else
      echo "$package_name $package_version is already published"
    fi
    echo ""
  done < <(find . -not -path '*/.*' -name 'Cargo.toml' -print0)
}

find_crate_projects() {
  local parent_dir
  for config_file in **/Cargo.lock; do
    echo "Found Cargo.lock file $(realpath "$config_file")"
    parent_dir="$(dirname "$config_file")"

    pushd "$parent_dir" >/dev/null || {
      echo "Failed to navigate to $parent_dir"
      exit 1
    }

    try_publishing_crate "$@"

    popd >/dev/null || {
      echo "Failed to navigate from $parent_dir"
      exit 1
    }
  done
}

login_to_crates_io() {
  if [[ " $* " == *" publish "* ]]; then
    export CARGO_HOME
    cargo login --token "$CARGO_REGISTRY_TOKEN"
  fi
}

check_inputs() {
  if [[ $# -ne 1 ]]; then
    echo "I need an argument: dryrun or publish"
    exit 1
  fi
}

check_inputs "$@"
login_to_crates_io "$@"
find_crate_projects "$@"
