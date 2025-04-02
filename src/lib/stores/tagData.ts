import { writable } from 'svelte/store';

// Instrument tags organized by categories
export const instrumentTags = writable([
  // String instruments
  'Acoustic Guitar', 'Electric Guitar', 'Bass Guitar', 'Violin', 'Viola', 'Cello', 'Double Bass', 'Harp', 'Banjo', 'Mandolin', 'Ukulele',
  
  // Keyboard instruments
  'Piano', 'Synthesizer', 'Organ', 'Electric Piano', 'Harpsichord', 'Accordion', 'Keytar',
  
  // Wind instruments
  'Flute', 'Clarinet', 'Saxophone', 'Trumpet', 'Trombone', 'French Horn', 'Tuba', 'Oboe', 'Bassoon',
  
  // Percussion
  'Drums', 'Drum Kit', 'Percussion', 'Congas', 'Bongos', 'Tambourine', 'Triangle', 'Maracas', 'Timpani', 'Xylophone', 'Vibraphone', 'Marimba',
  
  // Electronic & Other
  'DJ', 'Turntables', 'Sampler', 'Drum Machine', 'Vocoder', 'Theremin', 'Beatbox', 'Looper', 'Synth Bass', 'Lead Synth', 'Pad Synth'
]);

// Mood tags organized by categories
export const moodTags = writable([
  // Positive emotions
  'Happy', 'Uplifting', 'Joyful', 'Upbeat', 'Cheerful', 'Playful', 'Carefree', 'Optimistic', 'Bright', 'Empowering',
  
  // Peaceful emotions
  'Calm', 'Peaceful', 'Relaxing', 'Tranquil', 'Soothing', 'Dreamy', 'Ethereal', 'Ambient', 'Meditative', 'Gentle',
  
  // Energetic emotions
  'Energetic', 'Exciting', 'Dynamic', 'Powerful', 'Intense', 'Driving', 'Bold', 'Anthemic', 'Epic', 
  
  // Negative emotions
  'Sad', 'Melancholic', 'Somber', 'Dark', 'Moody', 'Tense', 'Anxious', 'Angry', 'Aggressive', 'Haunting',
  
  // Descriptive emotions
  'Nostalgic', 'Romantic', 'Dramatic', 'Mysterious', 'Quirky', 'Cinematic', 'Inspirational', 'Suspenseful', 'Whimsical', 'Emotional'
]);

// Function to convert an array of selected tags to a comma-separated string
export function tagsToString(tags: string[]): string {
  return tags.join(', ');
}

// Function to parse a comma-separated string into an array of tags
export function stringToTags(tagString: string): string[] {
  if (!tagString) return [];
  return tagString.split(',').map(tag => tag.trim()).filter(tag => tag.length > 0);
} 