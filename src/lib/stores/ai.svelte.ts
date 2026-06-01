class AIStore {
  messages = $state<{ role: 'user' | 'assistant'; content: string }[]>([]);
  isGenerating = $state(false);

  async sendMessage(_content: string) {
    this.isGenerating = true;
    // TODO: Implement AI chat in Phase 1
    this.isGenerating = false;
  }
}

export const aiStore = new AIStore();
