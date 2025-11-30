# Cursor AI Tutorial - YouTube Video Summarizer Implementation

**Source**: [Implement A Simple YouTube Video Summarizer Application using Cursor AI](https://medium.com/the-ai-forum/implement-a-simple-youtube-video-summarizer-application-using-cursor-ai-6b701027aec7) - Medium, September 27, 2024

**Author**: Plaban Nayak

## Overview

This tutorial demonstrates how to build a YouTube video summarizer application using Cursor AI. It serves as both a practical implementation guide and an introduction to Cursor AI's capabilities for code generation and development assistance.

## What Is Cursor AI?

Cursor AI is an innovative code editor that combines:

- **VS Code foundation**: Familiar interface and extensive ecosystem
- **AI capabilities**: Powered by models like OpenAI's ChatGPT and Claude
- **Context-aware assistance**: Understands your entire codebase
- **Intelligent code generation**: Goes beyond autocomplete to generate entire functions

**Key Point**: Cursor AI is a fork of Visual Studio Code, retaining the user-friendly interface while adding sophisticated AI tools.

## Key Features of Cursor AI

### 1. AI Code Completion

- Predicts multi-line edits
- Generates entire functions based on recent changes
- Writes complex code more quickly and accurately

### 2. Error Detection and Correction

- Continuously monitors code for errors
- Provides instant suggestions for fixes
- Proactive debugging approach

### 3. Natural Language Commands

- Interact using plain English
- Simplifies coding tasks
- Makes codebases accessible to non-experts

### 4. Dynamic Code Optimization

- Suggests improvements to existing code
- Refactoring recommendations
- Simplifies complex structures

### 5. Interactive Chat Features

- Query the codebase directly
- Ask questions about functions or variables
- Get context-aware suggestions

## Installation

### System Requirements

- Available for **Linux, Windows, and macOS**
- Free download from [trycursor.com](https://www.trycursor.com)

### Initial Configuration

After installation, configure these options:

1. **Keyboard**: Configure keyboard shortcuts (default: VS Code shortcuts recommended)
2. **Language for AI**: Option to use non-English languages for AI interaction
3. **Codebase-wide**: Enable to allow AI to understand entire codebase context
4. **Add terminal command**: Allows running Cursor AI from terminal (if installed)

## Essential Cursor AI Shortcuts

### Cursor Composer (`CTRL + I`)

- **Purpose**: Modify multiple files at once or generate entire applications
- **Use Case**: Creating complete features or refactoring across files

### AI Pane (`CTRL + L`)

- **Purpose**: Interact with Cursor, ask questions about codebase
- **Use Case**: Getting suggestions and understanding code

### Inline Code Editing (`CTRL + K`)

- **Purpose**: Ask AI to edit codebase directly
- **Use Case**: Quick code modifications in place

**Note**: All inline editing can be closed by pressing `ESC`.

**Windows Commands**: The above commands are for Windows. On macOS, use `CMD` instead of `CTRL`.

## Available Models

### Default Models

- `GPT-4o`
- `GPT-4`
- `Claude 3.5 Sonnet`
- `cursor-small` - Cursor's custom model (faster, unlimited access, but less capable than GPT-4)

### Adding Models

- Go to: `Cursor Settings` > `Models` > `Model Names`
- Add additional models as needed

### Long Context Models

For long context chat (limited to models supporting long context):

- `gpt-4o-128k`
- `gemini-1.5-flash-500k`
- `claude-3-haiku-200k`
- `claude-3-sonnet-200k`
- `claude-3-5-sonnet-200k`

**Current Chat Limit**: 20,000 tokens

## Tutorial: YouTube Video Summarizer

### Project Overview

Build a FastAPI web application that:

- Takes a YouTube video URL as input
- Extracts the video transcript
- Summarizes the transcript using Groq's Llama 3.1 model
- Displays results in a dark-mode UI using DaisyUI

### Implementation Steps

#### Step 1: Use Cursor Composer

1. Open Cursor Composer (`CTRL + I`)
2. Enter the project specification:

```
Create a FastAPI WebAPP in python which takes in a Youtube videourl and summarizes it using Groq Llama3 Model- use darkmode and daisy UI
```

1. Cursor AI generates the complete application
2. Press "Accept All" to accept the generated code

#### Step 2: Generated Code Structure

The generated application includes:

**Backend (FastAPI)**:

- Video URL extraction from YouTube links
- Transcript fetching using `youtube_transcript_api`
- Text summarization using Groq API (Llama 3.1-70B model)
- REST API endpoint for summarization

**Frontend (HTML/CSS/JavaScript)**:

- Dark mode UI with DaisyUI
- Form for entering YouTube URL
- Display area for summary results
- Responsive design

#### Step 3: Required Packages

Install dependencies:

```bash
pip install fastapi pydantic youtube_transcript_api groq python-dotenv uvicorn
```

**Package Breakdown**:

- `fastapi`: Web framework
- `pydantic`: Data validation
- `youtube_transcript_api`: Extract YouTube transcripts
- `groq`: Groq API client for Llama models
- `python-dotenv`: Environment variable management
- `uvicorn`: ASGI server for FastAPI

#### Step 4: Environment Setup

Create a `.env` file:

```env
GROQ_API_KEY=your_groq_api_key_here
```

#### Step 5: Code Features

**Video ID Extraction**:

```python
def get_video_id(url):
    # Extract video ID from YouTube URL
    if "youtu.be" in url:
        return url.split("/")[-1]
    elif "youtube.com" in url:
        return url.split("v=")[1].split("&")[0]
    else:
        raise ValueError("Invalid YouTube URL")
```

**Transcript Fetching**:

```python
def get_transcript(video_id):
    transcript = YouTubeTranscriptApi.get_transcript(video_id)
    return " ".join([entry['text'] for entry in transcript])
```

**Text Summarization**:

```python
def summarize_text(text):
    prompt = f"Summarize the following YouTube video transcript:\n\n{text}\n\nSummary:"

    response = client.chat.completions.create(
        model="llama-3.1-70b-versatile",
        messages=[
            {"role": "system", "content": "You are a helpful assistant that summarizes YouTube videos."},
            {"role": "user", "content": prompt}
        ],
        max_tokens=500
    )
    return response.choices[0].message.content
```

#### Step 6: Configuration Options

**Model Configuration**:

- **Model**: `llama-3.1-70b-versatile`
- **Max Tokens**: 500 (adjustable)
- **Temperature**: Configurable (affects creativity/randomness)

#### Step 7: Running the Application

1. **Activate virtual environment** (recommended):

```bash
python -m venv venv
source venv/bin/activate  # On macOS/Linux

# OR

venv\Scripts\activate  # On Windows
```

1. **Install packages**:

```bash
pip install fastapi pydantic youtube_transcript_api groq python-dotenv uvicorn
```

1. **Run the application**:

```bash
python youtube_summarizer.py
```

1. **Access the web interface**:
   - Navigate to: `http://localhost:8000`
   - Enter a YouTube URL
   - Click "Summarize"
   - View the generated summary

#### Step 8: Asking Cursor AI for Help

You can ask Cursor AI directly for package requirements:

**Query**: *"What packages do I need to install in order to run the youtube_summarizer script"*

**Response**: Cursor AI lists all required packages with installation commands.

### Example Usage

**Input YouTube URL**: `https://www.youtube.com/watch?v=gqUQbjsYZLQ`

**Generated Summary**: The application extracts the transcript and generates a comprehensive summary covering:

- Key topics discussed
- Main points and takeaways
- Detailed explanations of concepts

## Benefits of Using Cursor AI

### 1. Increased Productivity

- Automates repetitive tasks
- Provides intelligent suggestions
- Completes projects faster with fewer errors

### 2. Enhanced Collaboration

- Real-time interaction capabilities
- Easier sharing of insights and queries
- Improved team communication about codebase

### 3. Learning Tool for Beginners

- Natural language processing makes coding more intuitive
- Helps learn coding concepts
- Accessible to new programmers

## Key Takeaways for This Project

### Using Cursor AI with TWS Automated Trading Project

1. **Codebase-Wide Understanding**: Enable codebase-wide context so Cursor AI understands:
   - TWS API integration patterns
   - Project architecture
   - Existing code conventions

2. **Quick Prototyping**: Use Cursor Composer (`CTRL + I`) to:
   - Generate test implementations
   - Create boilerplate code for new features
   - Refactor existing code

3. **Documentation Assistance**: Use AI Pane (`CTRL + L`) to:
   - Ask questions about existing code
   - Understand complex TWS API patterns
   - Get suggestions for improvements

4. **Inline Editing**: Use `CTRL + K` for:
   - Quick bug fixes
   - Code improvements
   - Refactoring suggestions

### Best Practices

1. **Plan Before Coding**: Visualize and sketch ideas before using Cursor AI
2. **Use Documentation Tags**: Tag `@docs` to help Cursor AI access latest information
3. **Iterative Development**: Accept generated code, then refine iteratively
4. **Ask Follow-up Questions**: Use chat features to understand generated code
5. **Review Generated Code**: Always review and test AI-generated code

## Example Workflow

### Implementing a New TWS API Feature

1. **Plan**: Sketch out the feature requirements
2. **Document**: Add relevant TWS API documentation using `@docs`
3. **Compose**: Use Cursor Composer (`CTRL + I`) to generate initial implementation
4. **Review**: Check generated code for correctness
5. **Test**: Run tests to verify functionality
6. **Refine**: Use inline editing (`CTRL + K`) for improvements
7. **Document**: Add comments and documentation

## Related Resources

- [Cursor AI Official Site](https://www.trycursor.com) - Download and documentation
- [Cursor Setup Guide](research/integration/CURSOR_SETUP.md) - Project-specific Cursor setup
- [Cursor Docs Usage](research/integration/CURSOR_DOCS_USAGE.md) - Using `@docs` in Cursor
- [Cursor Recommendations](CURSOR_RECOMMENDATIONS.md) - Best practices for this project

## Technical Stack Reference

### Technologies Used in Tutorial

- **FastAPI**: Modern Python web framework
- **Groq API**: High-performance inference for LLMs
- **Llama 3.1**: Meta's open-source language model (70B parameters)
- **YouTube Transcript API**: Python library for extracting transcripts
- **DaisyUI**: Tailwind CSS component library
- **Uvicorn**: ASGI server for FastAPI

### Similar Applications

The YouTube summarizer pattern could be adapted for:

- Document summarization
- Code documentation generation
- Meeting note summarization
- Research paper summaries

## Conclusion

This tutorial demonstrates how Cursor AI can accelerate development by:

- Generating complete applications from natural language descriptions
- Providing context-aware code suggestions
- Facilitating rapid prototyping and iteration
- Making complex development tasks more accessible

For the TWS Automated Trading project, Cursor AI's capabilities can be leveraged for:

- Generating boilerplate code for new features
- Understanding complex API integrations
- Creating test implementations
- Refactoring existing code
- Generating documentation

The key is to use Cursor AI as a powerful assistant while maintaining code quality through review, testing, and iteration.

---

*Last Updated: Based on Medium article from September 27, 2024*
