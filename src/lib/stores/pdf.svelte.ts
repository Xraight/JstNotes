import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { readFile } from '@tauri-apps/plugin-fs';
import type { PdfMetadata, PdfAnnotation, AnnotationInput } from '../types';

class PdfStore {
  pdfs = $state<PdfMetadata[]>([]);
  activePdf = $state<PdfMetadata | null>(null);
  annotations = $state<PdfAnnotation[]>([]);
  pdfData = $state<Uint8Array | null>(null);
  isOpen = $state(false);

  async importPdf(): Promise<PdfMetadata | null> {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'PDF', extensions: ['pdf'] }],
    });
    if (!selected) return null;

    try {
      const meta = await invoke<PdfMetadata>('import_pdf', { filePath: selected });
      this.pdfs.push(meta);
      await this.openPdf(meta);
      return meta;
    } catch (e) {
      console.error('Failed to import PDF:', e);
      return null;
    }
  }

  async openPdf(meta: PdfMetadata) {
    this.activePdf = meta;
    this.isOpen = true;
    this.annotations = await this.loadAnnotations(meta.id);

    try {
      const data = await readFile(meta.file_path);
      console.log('PDF file read, size:', data.length, 'bytes');
      this.pdfData = data;
    } catch (e) {
      console.error('Failed to read PDF file at', meta.file_path, e);
      this.pdfData = null;
    }
  }

  closePdf() {
    this.activePdf = null;
    this.annotations = [];
    this.pdfData = null;
    this.isOpen = false;
  }

  async loadAnnotations(pdfId: string): Promise<PdfAnnotation[]> {
    try {
      return await invoke<PdfAnnotation[]>('get_annotations', { pdfId });
    } catch {
      return [];
    }
  }

  async saveAnnotation(input: AnnotationInput): Promise<PdfAnnotation | null> {
    try {
      const annotation = await invoke<PdfAnnotation>('save_annotation', { input });
      this.annotations.push(annotation);
      return annotation;
    } catch (e) {
      console.error('Failed to save annotation:', e);
      return null;
    }
  }

  async deleteAnnotation(annotationId: string) {
    try {
      await invoke('delete_annotation', { annotationId });
      this.annotations = this.annotations.filter(a => a.id !== annotationId);
    } catch (e) {
      console.error('Failed to delete annotation:', e);
    }
  }

  async linkToNote(noteId: string, pdfId: string) {
    try {
      await invoke('link_pdf_to_note', { noteId, pdfId });
    } catch (e) {
      console.error('Failed to link PDF to note:', e);
    }
  }

  async unlinkFromNote(noteId: string, pdfId: string) {
    try {
      await invoke('unlink_pdf_from_note', { noteId, pdfId });
    } catch (e) {
      console.error('Failed to unlink PDF from note:', e);
    }
  }

  async getPdfsForNote(noteId: string): Promise<PdfMetadata[]> {
    try {
      return await invoke<PdfMetadata[]>('get_pdfs_for_note', { noteId });
    } catch {
      return [];
    }
  }

  async deletePdf(pdfId: string) {
    try {
      await invoke('delete_pdf', { pdfId });
      this.pdfs = this.pdfs.filter(p => p.id !== pdfId);
      if (this.activePdf?.id === pdfId) {
        this.closePdf();
      }
    } catch (e) {
      console.error('Failed to delete PDF:', e);
    }
  }

  toggle() {
    if (this.isOpen) {
      this.closePdf();
    } else {
      this.importPdf();
    }
  }
}

export const pdfStore = new PdfStore();
