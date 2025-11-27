import { Component, inject, signal } from '@angular/core';
import { DialogRef } from '@ngneat/dialog';

@Component({
  selector: 'app-upload-files',
  imports: [],
  templateUrl: './upload-files.html',
  styleUrl: './upload-files.css',
})
export class UploadFiles {
   isDragging = signal(false);
  dialogRef = inject(DialogRef);

  onDragOver(event: DragEvent): void {
    event.preventDefault();
    this.isDragging.set(true);
  }

  onDragLeave(event: DragEvent): void {
    event.preventDefault();
    this.isDragging.set(false);
  }

  onUploadFiles(event: Event): void {
    const input = event.target as HTMLInputElement;
    
    if (input.files && input.files.length > 0) {    
      
      this.dialogRef.close({
          status: 'files',
          files: input.files
      });
    }
  }

  onDrop(event: DragEvent): void {
    event.preventDefault();
    this.isDragging.set(false);
    
    const files = event.dataTransfer?.files;
    
    if (files && files.length > 0) {
      this.dialogRef.close({
        status: 'files',
        files,
      });
    }
  }
}