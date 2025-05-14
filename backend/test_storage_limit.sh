#!/bin/bash
# Script to test the 500MB storage limit with high precision reporting

# Basic setup
HOST=http://localhost:3000
EMAIL=CFdefence@gmail.com
PASSWORD=MyPassword
COOKIE_FILE=auth_cookie.txt

echo "Testing 500MB storage limit functionality with precision measurement..."

# Login to get auth cookie
echo "Logging in..."
curl -s -X POST $HOST/api/users/login \
  -H "Content-Type: application/json" \
  -d "{\"email\":\"$EMAIL\",\"password\":\"$PASSWORD\"}" \
  -c $COOKIE_FILE > /dev/null

# Get initial storage state
echo "Checking initial storage usage..."
INITIAL_STORAGE=$(curl -s $HOST/api/users/storage -b $COOKIE_FILE)
echo "$INITIAL_STORAGE" | python3 -c "import sys, json; data=json.load(sys.stdin); print(f'Initial usage: {data[\"storage_bytes\"] or 0} bytes, {data[\"storage_bytes_formatted\"][\"kb\"]} KB, {data[\"storage_bytes_formatted\"][\"mb\"]} MB')"
echo "$INITIAL_STORAGE" | python3 -c "import sys, json; data=json.load(sys.stdin); print(f'Storage percentage: {data[\"storage_percentage\"]}% of 500MB')"

# Test 1: Create a document with just a few characters
echo -e "\n*** TEST 1: Very Small Text (10 bytes) ***"
CONTENT="HelloWorld"  # 10 bytes
echo "Creating a document with just '$CONTENT' (10 bytes)..."
RESULT=$(curl -s -X POST $HOST/api/document \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"Tiny Storage Test\",\"content\":\"$CONTENT\",\"created_at\":\"$(date -u +"%Y-%m-%dT%H:%M:%SZ")\",\"updated_at\":\"$(date -u +"%Y-%m-%dT%H:%M:%SZ")\"}" \
  -b $COOKIE_FILE)

if [[ $RESULT == *"error"* ]]; then
  echo "Error creating document: $RESULT"
else
  echo "Document created successfully"
  
  # Get document ID for later
  DOC_ID=$(echo $RESULT | python3 -c "import sys, json; print(json.load(sys.stdin)['id'])")
  
  # Check storage with high precision
  STORAGE=$(curl -s $HOST/api/users/storage -b $COOKIE_FILE)
  BYTES=$(echo "$STORAGE" | python3 -c "import sys, json; data=json.load(sys.stdin); print(data['storage_bytes_formatted']['bytes'])")
  BYTES_KB=$(echo "$STORAGE" | python3 -c "import sys, json; data=json.load(sys.stdin); print(data['storage_bytes_formatted']['kb'])")
  BYTES_MB=$(echo "$STORAGE" | python3 -c "import sys, json; data=json.load(sys.stdin); print(data['storage_bytes_formatted']['mb'])")
  PERCENTAGE=$(echo "$STORAGE" | python3 -c "import sys, json; data=json.load(sys.stdin); print(data['storage_percentage'])")
  
  echo "After adding 10 bytes:"
  echo "  Raw bytes: $BYTES bytes"
  echo "  As KB: $BYTES_KB KB"
  echo "  As MB: $BYTES_MB MB"
  echo "  Percentage of 500MB: $PERCENTAGE%"
  
  # Test 2: Update with a single additional character
  echo -e "\n*** TEST 2: Single Character Addition (1 byte) ***"
  # Update document with one extra character
  CONTENT="${CONTENT}A"  # Add one character (1 byte)
  echo "Updating document by adding just one character 'A' (1 byte)..."
  RESULT=$(curl -s -X PUT $HOST/api/document/$DOC_ID \
    -H "Content-Type: application/json" \
    -d "{\"name\":\"Tiny Storage Test\",\"content\":\"$CONTENT\",\"updated_at\":\"$(date -u +"%Y-%m-%dT%H:%M:%SZ")\"}" \
    -b $COOKIE_FILE)
  
  if [[ $RESULT == *"error"* ]]; then
    echo "Error updating document: $RESULT"
  else
    echo "Document updated successfully"
    
    # Check storage for single byte change
    STORAGE=$(curl -s $HOST/api/users/storage -b $COOKIE_FILE)
    NEW_BYTES=$(echo "$STORAGE" | python3 -c "import sys, json; data=json.load(sys.stdin); print(data['storage_bytes_formatted']['bytes'])")
    NEW_BYTES_KB=$(echo "$STORAGE" | python3 -c "import sys, json; data=json.load(sys.stdin); print(data['storage_bytes_formatted']['kb'])")
    NEW_BYTES_MB=$(echo "$STORAGE" | python3 -c "import sys, json; data=json.load(sys.stdin); print(data['storage_bytes_formatted']['mb'])")
    NEW_PERCENTAGE=$(echo "$STORAGE" | python3 -c "import sys, json; data=json.load(sys.stdin); print(data['storage_percentage'])")
    
    DIFF=$((NEW_BYTES - BYTES))
    
    echo "After adding 1 more byte:"
    echo "  Raw bytes: $NEW_BYTES bytes (+$DIFF)"
    echo "  As KB: $NEW_BYTES_KB KB"
    echo "  As MB: $NEW_BYTES_MB MB"
    echo "  Percentage of 500MB: $NEW_PERCENTAGE%"
  fi
fi

# Test 3: Test larger document for comparison
echo -e "\n*** TEST 3: Larger Document (5MB) ***"
CONTENT=$(python3 -c "print('B' * 5 * 1024 * 1024)")
echo "Creating a document with 5MB of data..."
RESULT=$(curl -s -X POST $HOST/api/document \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"Larger Storage Test\",\"content\":\"$CONTENT\",\"created_at\":\"$(date -u +"%Y-%m-%dT%H:%M:%SZ")\",\"updated_at\":\"$(date -u +"%Y-%m-%dT%H:%M:%SZ")\"}" \
  -b $COOKIE_FILE)

if [[ $RESULT == *"error"* ]]; then
  echo "Error creating document: $RESULT"
else
  echo "Document created successfully"
  
  # Check storage again
  STORAGE=$(curl -s $HOST/api/users/storage -b $COOKIE_FILE)
  echo "After adding 5MB:"
  echo "$STORAGE" | python3 -c "import sys, json; data=json.load(sys.stdin); print(f'  Raw bytes: {data[\"storage_bytes_formatted\"][\"bytes\"]} bytes')"
  echo "$STORAGE" | python3 -c "import sys, json; data=json.load(sys.stdin); print(f'  As KB: {data[\"storage_bytes_formatted\"][\"kb\"]} KB')"
  echo "$STORAGE" | python3 -c "import sys, json; data=json.load(sys.stdin); print(f'  As MB: {data[\"storage_bytes_formatted\"][\"mb\"]} MB')"
  echo "$STORAGE" | python3 -c "import sys, json; data=json.load(sys.stdin); print(f'  Percentage of 500MB: {data[\"storage_percentage\"]}%')"
fi

# Clean up
echo -e "\nCleaning up..."
rm -f $COOKIE_FILE

echo "Test completed!" 