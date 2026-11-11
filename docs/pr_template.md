# Add support for Claude 3.7 Sonnet

## Description
This PR adds support for the new Claude 3.7 Sonnet model from Anthropic. The model offers enhanced reasoning capabilities, an improved context window, and better performance compared to previous Claude models.

## Changes
- Added `CLAUDE_3_7_SONNET` constant in `anthropic/completion.rs`
- Set the constant value to "claude-3-7-sonnet-latest" following Anthropic's naming convention
- Placed the constant in logical order with other Claude model constants

## Testing
- Verified that the model naming follows Anthropic's standard format
- Confirmed compatibility with existing Anthropic provider implementation
- No functional changes to the code, only adding a new model identifier constant

## Documentation
- No documentation changes required as this is a straightforward addition of a new model constant

## Additional Notes
This is a minimal change to add support for the latest Claude model, allowing users to leverage the improved capabilities of Claude 3.7 Sonnet in their applications.

## Related Issues
Closes: #1234 (Replace with actual issue number if applicable) 