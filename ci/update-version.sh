UPDATE_KIND="patch"
COMMIT_CHANGES="false"

while [[ $# -gt 0 ]]; do

  case "$1" in
    "--kind"|"-k")
      UPDATE_KIND="$2"
      shift
      shift
    ;;

    "--commit"|"-c")
      COMMIT_CHANGES="$2"
      shift
      shift
    ;;
  esac

done

if [ -z "$UPDATE_KIND" ]; then
  UPDATE_KIND="patch"
fi;

if [ ! "$UPDATE_KIND" == "patch" ] && [ ! "$UPDATE_KIND" == "minor" ] && [ ! "$UPDATE_KIND" == "major" ]; then
  echo "Unsupported version update kind: $UPDATE_KIND"
  exit 4
fi;

CURRENT=$(cat Cargo.toml | grep "^version" | grep -o -E "[0-9]+\.[0-9]+\.[0-9]+")

echo "> Current version: $CURRENT"
echo "> Version update: $UPDATE_KIND"

# compute new version
MAJOR=$(echo "$CURRENT" | grep -o "^[0-9]")
MINOR=$(echo "$CURRENT" | grep -o "\.[0-9]\." | tr -d '.')
PATCH=$(echo "$CURRENT" | grep -o "[0-9]$")

case $UPDATE_KIND in
  "patch")
    PATCH=$(($PATCH + 1))
  ;;

  "minor")
    MINOR=$(($MINOR + 1))
  ;;

  "major")
    MAJOR=$(($MAJOR + 1))
  ;;
esac

NEW="$MAJOR.$MINOR.$PATCH"

echo "> Updated version: $NEW"


# update Cargo.toml
sed -i "s/^version = \"$CURRENT\"/version = \"$NEW\"/g" Cargo.toml

# update in README.md
sed -i "s/terror = \"$CURRENT\"/terror = \"$NEW\"/g" README.md

if [ "$COMMIT_CHANGES" == "true" ]; then
  echo "> Committing changes"

  git config user.name "terror-release-chore"
  git config user.email "terror-release-chore@gh.io"

  MESSAGE="[release] updating $CURRENT > $NEW"
  git add .
  git commit -m "$MESSAGE"
  git push

  git tag -a "$NEW" -m "$MESSAGE"
  git push --tags

fi