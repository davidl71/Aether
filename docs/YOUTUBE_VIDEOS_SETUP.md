# YouTube Videos Setup Guide for NotebookLM

This guide provides step-by-step instructions for adding the YouTube videos to NotebookLM and creating summaries.

## Videos to Add

### Total Videos: 8 videos

1. TWS API: Essential Components (1 video)
2. Trading & Options: 7 videos (including 2 playlists)

## Quick Setup Instructions

### Option 1: Add All Videos to One Notebook (Recommended)

1. **Create Notebook in NotebookLM**:
   - Go to [notebooklm.google.com](https://notebooklm.google.com)
   - Click **"+ New"** to create a new notebook
   - Name it: **"TWS Automated Trading - All Videos"**

2. **Add All Videos**:
   - Click **"+ Add source"**
   - Select **"YouTube"** or **"Website"**
   - Add each video URL one by one:
     - <https://www.youtube.com/watch?v=n-9bdREECTQ>
     - <https://www.youtube.com/watch?v=5moyX0qwkCA>
     - <https://www.youtube.com/watch?v=hJ7ewxQVhJw&list=PLePBf4ZtCKhpovbIfv85Ks3-rvyNtAwsg&index=2>
     - <https://www.youtube.com/watch?v=4zpYhHn5p90&list=PLePBf4ZtCKhqBGXyB1v4xvZiMju6NQlIw&index=2>
     - <https://www.youtube.com/watch?v=rC02897uiuc&list=PLePBf4ZtCKhpovbIfv85Ks3-rvyNtAwsg&index=2>
     - <https://www.youtube.com/watch?v=ZxwdTgMY44g&list=PLePBf4ZtCKhqBGXyB1v4xvZiMju6NQlIw&index=2>
     - <https://www.youtube.com/watch?v=ICZH89GdUGQ&list=PLePBf4ZtCKhqBGXyB1v4xvZiMju6NQlIw&index=2>
     - <https://www.youtube.com/watch?v=W6OJy32sE_g&list=PLePBf4ZtCKhqBGXyB1v4xvZiMju6NQlIw&index=2>

3. **Wait for Processing**: Each video may take 2-5 minutes to process

4. **Share Notebook**:
   - Click **"⚙️ Share"** (top right)
   - Select **"Anyone with link"**
   - Click **"Copy link"**
   - Save the link

5. **Add to Library**:
   - Return to Cursor and say:

   ```
   "Add [paste-the-link-here] to library tagged 'youtube, tws-api, trading, options, tutorial'"
   ```

### Option 2: Add Videos by Category (Organized)

#### Notebook 1: TWS API Videos

1. Create notebook: **"TWS API Videos"**
2. Add video:
   - <https://www.youtube.com/watch?v=n-9bdREECTQ>
3. Share and add to library with tag: `'youtube, tws-api, tutorial'`

#### Notebook 2: Trading & Options Videos

1. Create notebook: **"Trading & Options Videos"**
2. Add all 7 trading videos
3. Share and add to library with tag: `'youtube, trading, options, tutorial'`

#### Notebook 3: Playlist 1 Videos

1. Create notebook: **"Trading Playlist 1"**
2. Add videos from playlist 1:
   - <https://www.youtube.com/watch?v=hJ7ewxQVhJw&list=PLePBf4ZtCKhpovbIfv85Ks3-rvyNtAwsg&index=2>
   - <https://www.youtube.com/watch?v=rC02897uiuc&list=PLePBf4ZtCKhpovbIfv85Ks3-rvyNtAwsg&index=2>
3. Share and add to library with tag: `'youtube, trading, playlist'`

#### Notebook 4: Playlist 2 Videos

1. Create notebook: **"Trading Playlist 2"**
2. Add videos from playlist 2:
   - <https://www.youtube.com/watch?v=4zpYhHn5p90&list=PLePBf4ZtCKhqBGXyB1v4xvZiMju6NQlIw&index=2>
   - <https://www.youtube.com/watch?v=ZxwdTgMY44g&list=PLePBf4ZtCKhqBGXyB1v4xvZiMju6NQlIw&index=2>
   - <https://www.youtube.com/watch?v=ICZH89GdUGQ&list=PLePBf4ZtCKhqBGXyB1v4xvZiMju6NQlIw&index=2>
   - <https://www.youtube.com/watch?v=W6OJy32sE_g&list=PLePBf4ZtCKhqBGXyB1v4xvZiMju6NQlIw&index=2>
3. Share and add to library with tag: `'youtube, trading, playlist'`

### Option 3: Add Entire Playlists (Best Context)

If NotebookLM supports adding entire playlists:

1. **Create Notebook**: **"Trading Playlists"**
2. **Add Playlist URLs**:
   - Playlist 1: `https://www.youtube.com/playlist?list=PLePBf4ZtCKhpovbIfv85Ks3-rvyNtAwsg`
   - Playlist 2: `https://www.youtube.com/playlist?list=PLePBf4ZtCKhqBGXyB1v4xvZiMju6NQlIw`
3. **Add Individual Videos**:
   - <https://www.youtube.com/watch?v=n-9bdREECTQ>
   - <https://www.youtube.com/watch?v=5moyX0qwkCA>
4. **Share and add to library**

## After Adding Videos to NotebookLM

### Step 1: Verify Videos Are Processed

Check that all videos are processed in NotebookLM:

- Videos should show as "Ready" or "Processed"
- If any fail, try re-adding them individually

### Step 2: Add Notebook to Library

Once the notebook is shared, add it to the library:

```
"Add [notebook-link] to library tagged 'youtube, tws-api, trading, options, tutorial'"
```

### Step 3: Summarize Videos

After adding to library, you can summarize videos:

#### Summarize Individual Videos

```
"Summarize the TWS API video (https://www.youtube.com/watch?v=n-9bdREECTQ) and save to docs/video-summaries/tws-api-essential-components.md"
```

#### Summarize All Videos

```
"Research all videos in the notebook and create summaries for each in docs/video-summaries/"
```

#### Research Specific Topics

```
"Research TWS API architecture in NotebookLM using the video notebook"
```

## Video List for Quick Reference

### TWS API Videos

1. **Essential Components of TWS API Programs**
   - URL: <https://www.youtube.com/watch?v=n-9bdREECTQ>
   - Topic: TWS API architecture, EClient, EWrapper

### Trading & Options Videos

1. **Video 1**
   - URL: <https://www.youtube.com/watch?v=5moyX0qwkCA>

2. **Video 2** (Playlist 1)
   - URL: <https://www.youtube.com/watch?v=hJ7ewxQVhJw&list=PLePBf4ZtCKhpovbIfv85Ks3-rvyNtAwsg&index=2>

3. **Video 3** (Playlist 2)
   - URL: <https://www.youtube.com/watch?v=4zpYhHn5p90&list=PLePBf4ZtCKhqBGXyB1v4xvZiMju6NQlIw&index=2>

4. **Video 4** (Playlist 1)
   - URL: <https://www.youtube.com/watch?v=rC02897uiuc&list=PLePBf4ZtCKhpovbIfv85Ks3-rvyNtAwsg&index=2>

5. **Video 5** (Playlist 2)
   - URL: <https://www.youtube.com/watch?v=ZxwdTgMY44g&list=PLePBf4ZtCKhqBGXyB1v4xvZiMju6NQlIw&index=2>

6. **Video 6** (Playlist 2)
   - URL: <https://www.youtube.com/watch?v=ICZH89GdUGQ&list=PLePBf4ZtCKhqBGXyB1v4xvZiMju6NQlIw&index=2>

7. **Video 7** (Playlist 2)
   - URL: <https://www.youtube.com/watch?v=W6OJy32sE_g&list=PLePBf4ZtCKhqBGXyB1v4xvZiMju6NQlIw&index=2>

## Playlist Information

### Playlist 1: PLePBf4ZtCKhpovbIfv85Ks3-rvyNtAwsg

- **Playlist URL**: <https://www.youtube.com/playlist?list=PLePBf4ZtCKhpovbIfv85Ks3-rvyNtAwsg>
- **Videos in this collection**: 2 videos (Video 2, Video 4)
- **Note**: Consider adding entire playlist for full context

### Playlist 2: PLePBf4ZtCKhqBGXyB1v4xvZiMju6NQlIw

- **Playlist URL**: <https://www.youtube.com/playlist?list=PLePBf4ZtCKhqBGXyB1v4xvZiMju6NQlIw>
- **Videos in this collection**: 4 videos (Video 3, Video 5, Video 6, Video 7)
- **Note**: Consider adding entire playlist for full context

## Next Steps

1. **Add videos to NotebookLM** (choose one of the options above)
2. **Share the notebook** and get the link
3. **Add to library** using the Cursor chat command
4. **Summarize videos** as needed
5. **Update documentation** with video summaries

## See Also

- [YouTube Videos Documentation](YOUTUBE_VIDEOS.md) - Complete video tracking
- [NotebookLM Usage Guide](NOTEBOOKLM_USAGE.md) - Detailed NotebookLM usage
- [NotebookLM Setup Guide](NOTEBOOKLM_SETUP_GUIDE.md) - General setup instructions
- [Documentation Index](DOCUMENTATION_INDEX.md) - Complete documentation index
