#!/bin/bash
# Guardian OS Repository Initial Setup

echo "Guardian OS Repository Setup"
echo "============================"

# Check if we're in the right directory
if [ ! -f "Makefile" ]; then
    echo "Error: Not in the Guardian OS directory"
    exit 1
fi

# Configure git (if not already configured)
git config user.name "David Smith" 2>/dev/null || true
git config user.email "your-email@example.com" 2>/dev/null || true

# Add all files to git
echo "Adding files to git..."
git add -A

# Show status
echo ""
echo "Git Status:"
git status

echo ""
echo "Repository is ready for initial commit!"
echo ""
echo "To commit and push to GitHub, run:"
echo "  git commit -m 'Initial commit: Guardian OS with Supabase integration'"
echo "  git push -u origin main"
echo ""
echo "Or use GitHub Desktop to commit and push these changes."
