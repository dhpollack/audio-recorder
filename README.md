# Audio Recorder with Whisper Transcription

A web-based audio recording application built with Rust, Leptos, Cloudflare Workers, Huggingface Candle, and WebAssembly that provides real-time audio recording and transcription using OpenAI's Whisper model locally in your browser.  This is primarily an example of how to use Leptos and Huggingface Candle to do on device machine learning in pure Rust.  There is also a version that uses tokio if you want to try this without a Cloudflare account

## Features

- 🎤 **Press-and-hold recording** - Record audio by pressing and holding the record button
- 📱 **Mobile-friendly** - Optimized for touch devices with proper touch event handling
- 🤖 **Whisper integration** - Automatic speech-to-text transcription using OpenAI's Whisper model
- 🔊 **Audio playback** - Listen to your recordings with built-in audio player
- ⚡ **Cloudflare Workers** - Deployable as a serverless application
- 🦀 **Rust-powered** - High performance and memory safety

## Tech Stack

- **Frontend**: Leptos (Rust framework for web apps)
- **Backend**: Cloudflare Workers or Tokio
- **Audio Processing**: Web Audio API, MediaRecorder API
- **Transcription**: OpenAI Whisper model
- **Styling**: CSS with mobile-first design

## Prerequisites

- Rust (latest stable)
- just
- cargo-leptos
- npx (comes with Node.js)

## Development

### Local Development

1. **Clone the repository**:
   ```bash
   git clone <repository-url>
   cd audio-recorder
   ```

2. **Install dependencies**:
   ```bash
   # TLDR
   just setup
   # set by set way
   cargo install just cargo-leptos
   ```

3. **Check to make sure the project properly builds**:
   ```bash
   just check
   ```

### Setup

Before running the application, you need to download the Whisper model weights:

```bash
just r2 download-weights
```

If you plan to deploy to Cloudflare Workers, you'll also need to create an R2 bucket and upload the files.  It should be noted that to enable R2, I had to enter a credit card, but would only be charged for usage above the free tier:

```bash
# Create the R2 bucket
just r2 create-bucket

# Upload the model weights to R2
just r2 upload
```

### Run locally

**Without Cloudflare (does not require a Cloudflare Account)**:
```bash
just dev-nocloudflare
```

**With Cloudflare Workers**:
```bash
just dev
```

5. **Visit site**:
```
open localhost:8786
```



### Cloudflare Workers Deployment

1. **Configure your Cloudflare account**:
   ```bash
   wrangler login
   ```

2. **Deploy to Cloudflare Workers**:
   ```bash
   just deploy
   ```

## Usage

### Recording Audio

1. **Load the Whisper Model**: Click the "Load Whisper Model" button to initialize the transcription engine
2. **Start Recording**: Press and hold the record button (or click and hold on desktop)
3. **Stop Recording**: Release the button to stop recording
4. **View Transcription**: The transcribed text will appear automatically
5. **Playback**: Use the audio player to listen to your recording

### Mobile Usage (Currently Broken)

The application is optimized for mobile devices:
- **Touch-friendly** buttons with proper sizing (44px minimum touch target)
- **Press-and-hold** functionality with proper touch event handling
- **Prevented default behaviors** to avoid scrolling/zooming during recording

## Project Structure

```
├── src/
│   ├── app.rs              # Main application component
│   ├── components/
│   │   ├── mod.rs          # Components module
│   │   └── media_recorder.rs # Audio recording component
│   ├── whisper/
│   │   ├── mod.rs          # Whisper module
│   │   ├── audio.rs        # Audio processing
│   │   └── languages.rs    # Language support
│   └── main.rs             # Entry point
├── style/
│   └── main.css            # Application styles
├── Cargo.toml              # Rust dependencies
└── wrangler.toml           # Cloudflare Workers configuration
```

## Configuration

### Audio Settings

The application uses the following default audio settings:
- **Sample Rate**: 16kHz (optimized for Whisper)
- **Channels**: Mono (1 channel)
- **Format**: WebM for recording, converted to WAV for processing

### Whisper Model

The Whisper model is loaded on-demand when the "Load Whisper Model" button is clicked. This helps reduce initial load time and resource usage.

## Browser Compatibility

- Chrome/Chromium 66+
- Firefox 76+
- Safari 14.1+
- Edge 79+

**Mobile Browser Support**:
- iOS Safari 14.5+
- Chrome Mobile 90+
- Firefox Mobile 86+

## Troubleshooting

### Common Issues

1. **Recording doesn't start on mobile**:
   - Ensure you're using a supported browser
   - Check that microphone permissions are granted
   - Try a longer press-and-hold

2. **Transcription not working**:
   - Make sure the Whisper model is loaded (click "Load Whisper Model")
   - Check browser console for any errors
   - Ensure stable internet connection for model loading

3. **Audio playback issues**:
   - Check browser audio permissions
   - Verify the audio format is supported by your browser

### Permissions

The application requires:
- **Microphone access** for audio recording
- **Internet connection** for Whisper model loading

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- [Leptos](https://github.com/leptos-rs/leptos) - Rust framework for building web applications
- [OpenAI Whisper](https://github.com/openai/whisper) - Speech recognition model
- [Cloudflare Workers](https://workers.cloudflare.com/) - Serverless deployment platform
- [Web Audio API](https://developer.mozilla.org/en-US/docs/Web/API/Web_Audio_API) - Audio processing capabilities
- [Huggingface Candle](https://github.com/huggingface/candle) - Minimalist ML Framework in Rust (wasm compatible)
- [Candle WASM Example Whisper](https://github.com/huggingface/candle/tree/main/candle-wasm-examples/whisper) - Adapted this example to use in leptos and Cloudflare Workers
