#!/bin/bash
## Usage: ./bump.bash [major|minor|patch]

VERSION=$(cargo read-manifest | jq -r .version)
MAJOR=$(echo "$VERSION" | cut -d . -f 1)
MINOR=$(echo "$VERSION" | cut -d . -f 2)
PATCH=$(echo "$VERSION" | cut -d . -f 3)
case $1 in
major)
    NEW_VERSION=$(printf "%s.0.0\n" $((MAJOR + 1)));;
minor)
    NEW_VERSION=$(printf "%s.%s.0\n" "$MAJOR" $((MINOR + 1)));;
patch)
    NEW_VERSION=$(printf "%s.%s.%s\n" "$MAJOR" "$MINOR" $((PATCH + 1)));;
*)
    echo invalid argument >&1
    exit 1
esac

function bump_changelog {
    CHANGELOG=$1

    awk '/## Unreleased/,0{next}{print}' "$CHANGELOG"

    cat - << EOS
## Unreleased

### Added

### Fixed

### Removed

### Changed

---

## $NEW_VERSION - $(date "+%Y-%m-%d")
EOS

    read -r -d '' PROG << 'EOS'
/###.*/ {
    section = $0
    next
}
{
    if (section) {
        print ""
        print section
        print ""
        section = ""
    }
    print
}
END{print ""}
EOS
    awk '/## Unreleased/,/---/' "$CHANGELOG" | awk '/./&&!/---/&&NR>1' | awk -v NEW_VERSION="$NEW_VERSION" "$PROG"
    awk '/---/,0{print}' "$CHANGELOG" | awk 'NR>2'
}

function bump_manifest {
    MANIFEST=$1
    awk -v NEW_VERSION="$NEW_VERSION" '/^version = .*/{printf "version = \"%s\"\n", NEW_VERSION;next}{print}' "$MANIFEST"
}

git switch -c release/"$NEW_VERSION"

# https://stackoverflow.com/a/73054135
# shellcheck disable=SC2094
cat <<< "$(bump_changelog CHANGELOG.md)" > CHANGELOG.md
# shellcheck disable=SC2094
cat <<< "$(bump_manifest msgpack-schema-impl/Cargo.toml)" > msgpack-schema-impl/Cargo.toml
# shellcheck disable=SC2094
cat <<< "$(bump_manifest msgpack-value/Cargo.toml)" > msgpack-value/Cargo.toml
# shellcheck disable=SC2094
cat <<< "$(bump_manifest Cargo.toml)" > Cargo.toml
sed -i "s/version = \"=$VERSION\"/version = \"=$NEW_VERSION\"/g" Cargo.toml

git add .
git commit -m "bump $NEW_VERSION"
git push -u origin HEAD

gh pr create --base master --title "Bump version $NEW_VERSION" --body "Bump $1 version" --label release
