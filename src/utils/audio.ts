// Audio announcement utility for Car Garage Ticket Calling
import { api } from "@/services/tauri";

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
    const now = ctx.currentTime;
    const osc1 = ctx.createOscillator();
    const gain1 = ctx.createGain();
    osc1.type = "sine";
    osc1.frequency.setValueAtTime(659.25, now);
    gain1.gain.setValueAtTime(0.4, now);
    gain1.gain.exponentialRampToValueAtTime(0.001, now + 0.3);
    osc1.connect(gain1);
    gain1.connect(ctx.destination);
    osc1.start(now);
    osc1.stop(now + 0.3);
    const osc2 = ctx.createOscillator();
    const gain2 = ctx.createGain();
    osc2.type = "sine";
    osc2.frequency.setValueAtTime(880, now + 0.15);
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

export function speakTicketCall(
  ticketNumber: string,
  bayLabel: string,
  callWord = "الزبون",
  repeatCount = 1,
) {
  if (typeof window === "undefined" || !("speechSynthesis" in window)) return;

  playChime();
  window.speechSynthesis.cancel();

  const ticketNumeric = parseInt(ticketNumber, 10);
  const ticketSpoken = isNaN(ticketNumeric) ? ticketNumber : ticketNumeric;
  const times = Math.max(1, Math.min(5, repeatCount));
  const text = callWord + " رقم " + String(ticketSpoken) + " إلى " + bayLabel;

  // Duck other apps before the first utterance
  void api.duckAudio().catch(() => {});

  let lastUtterance: SpeechSynthesisUtterance | null = null;

  for (let i = 0; i < times; i++) {
    const delay = 350 + i * 2500;
    setTimeout(() => {
      const utterance = new SpeechSynthesisUtterance(text);
      utterance.lang = "ar-SA";
      utterance.rate = 0.65;
      utterance.pitch = 1.0;
      const voices = window.speechSynthesis.getVoices();
      const arVoice = cachedArabicVoice || voices.find((v) => v.lang.startsWith("ar"));
      if (arVoice) utterance.voice = arVoice;

      // Restore other apps after the last utterance finishes
      if (i === times - 1) {
        utterance.onend = () => void api.unduckAudio().catch(() => {});
        utterance.onerror = () => void api.unduckAudio().catch(() => {});
        lastUtterance = utterance;
      }

      window.speechSynthesis.speak(utterance);
    }, delay);
  }

  // Safety net: restore after a generous timeout even if onend never fires
  const safetyMs = 350 + times * 2500 + 6000;
  setTimeout(() => {
    if (lastUtterance) void api.unduckAudio().catch(() => {});
  }, safetyMs);
}
