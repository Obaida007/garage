// Audio announcement utility for Car Garage Ticket Calling

let cachedArabicVoice: SpeechSynthesisVoice | null = null;

if (typeof window !== "undefined" && "speechSynthesis" in window) {
  const loadVoices = () => {
    const voices = window.speechSynthesis.getVoices();
    cachedArabicVoice = voices.find((v) => v.lang.startsWith("ar")) || null;
  };
  loadVoices();
  if (window.speechSynthesis.onvoiceschanged !== undefined) {
    window.speechSynthesis.onvoiceschanged = loadVoices;
  }
}

export function playChime() {
  try {
    const AudioContextClass = window.AudioContext || (window as any).webkitAudioContext;
    if (!AudioContextClass) return;
    const ctx = new AudioContextClass();
    
    // Play double chime (Ding-Dong)
    const now = ctx.currentTime;
    
    // First tone (Ding)
    const osc1 = ctx.createOscillator();
    const gain1 = ctx.createGain();
    osc1.type = "sine";
    osc1.frequency.setValueAtTime(659.25, now); // E5
    gain1.gain.setValueAtTime(0.4, now);
    gain1.gain.exponentialRampToValueAtTime(0.001, now + 0.3);
    osc1.connect(gain1);
    gain1.connect(ctx.destination);
    osc1.start(now);
    osc1.stop(now + 0.3);

    // Second tone (Dong)
    const osc2 = ctx.createOscillator();
    const gain2 = ctx.createGain();
    osc2.type = "sine";
    osc2.frequency.setValueAtTime(880, now + 0.15); // A5
    gain2.gain.setValueAtTime(0.5, now + 0.15);
    gain2.gain.exponentialRampToValueAtTime(0.001, now + 0.5);
    osc2.connect(gain2);
    gain2.connect(ctx.destination);
    osc2.start(now + 0.15);
    osc2.stop(now + 0.5);
  } catch (e) {
    console.warn("Audio Context error:", e);
  }
}

export function speakTicketCall(ticketNumber: string, bayId: number | string) {
  if (typeof window === "undefined" || !("speechSynthesis" in window)) {
    console.warn("Speech synthesis is not supported in this environment");
    return;
  }

  // Play chime first
  playChime();

  // Cancel any ongoing speech
  window.speechSynthesis.cancel();

  // Convert ticket number to integer so TTS reads it as a whole number (e.g. "071" → 71 → "واحد وسبعون")
  const ticketNumeric = parseInt(ticketNumber, 10);
  const ticketSpoken = isNaN(ticketNumeric) ? ticketNumber : ticketNumeric;

  // Format text: "الزبون رقم [رقم الزبون] إلى الحفرة [رقم الحفرة]"
  const text = `الزبون رقم ${ticketSpoken} إلى الحفرة ${bayId}`;

  // Delay speech slightly to let chime play cleanly
  setTimeout(() => {
    const utterance = new SpeechSynthesisUtterance(text);
    utterance.lang = "ar-SA";
    utterance.rate = 0.85; // Natural clear Arabic speaking rate
    utterance.pitch = 1.0;

    const voices = window.speechSynthesis.getVoices();
    const arVoice = cachedArabicVoice || voices.find((v) => v.lang.startsWith("ar"));
    if (arVoice) {
      utterance.voice = arVoice;
    }

    window.speechSynthesis.speak(utterance);
  }, 350);
}
