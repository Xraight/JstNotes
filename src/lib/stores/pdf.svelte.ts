import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { readFile } from '@tauri-apps/plugin-fs';
import type { PdfMetadata, PdfAnnotation, AnnotationInput, PdfReference, CreatePdfReferenceInput } from '../types';

class PdfStore {
  pdfs = $state<PdfMetadata[]>([]);
  activePdf = $state<PdfMetadata | null>(null);
  annotations = $state<PdfAnnotation[]>([]);
  pdfData = $state<Uint8Array | null>(null);
  isOpen = $state(false);
  linkedPdfIds = $state<string[]>([]);
  targetPage = $state<number | null>(null);

  async loadPdfs() {
    try {
      this.pdfs = await invoke<PdfMetadata[]>('list_pdfs');
    } catch (e) {
      console.error('Failed to load PDFs:', e);
    }
  }

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
    this.targetPage = null;
  }

  async openPdfAtPage(meta: PdfMetadata, page: number) {
    this.targetPage = page;
    await this.openPdf(meta);
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

  async updateAnnotationContent(annotationId: string, content: string | null): Promise<boolean> {
    try {
      await invoke('update_annotation_content', { annotationId, content });
      this.annotations = this.annotations.map(a =>
        a.id === annotationId ? { ...a, content } : a
      );
      return true;
    } catch (e) {
      console.error('Failed to update annotation:', e);
      return false;
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

  async loadLinkedPdfs(noteId: string) {
    try {
      this.linkedPdfIds = await invoke<string[]>('get_linked_pdf_ids', { noteId });
    } catch {
      this.linkedPdfIds = [];
    }
  }

  async createPdfReference(input: CreatePdfReferenceInput): Promise<PdfReference | null> {
    try {
      return await invoke<PdfReference>('create_pdf_reference', { input });
    } catch (e) {
      console.error('Failed to create PDF reference:', e);
      return null;
    }
  }

  async getPdfReferencesForNote(noteId: string): Promise<PdfReference[]> {
    try {
      return await invoke<PdfReference[]>('get_pdf_references_for_note', { noteId });
    } catch {
      return [];
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
