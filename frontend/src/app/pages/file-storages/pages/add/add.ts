import { Component, inject, signal } from '@angular/core';
import { FormBuilder, ReactiveFormsModule, Validators } from '@angular/forms';
import { Router } from '@angular/router';
import { FileStoragesService } from '@core/services/file-storages';
import { Store } from '@ngrx/store';
import { setLoading } from 'app/state/loading/loading.actions';
import Snackbar from 'awesome-snackbar';

@Component({
  selector: 'app-add',
  imports: [ReactiveFormsModule],
  templateUrl: './add.html',
  styleUrl: './add.css',
})
export class Add {
  fileStoragesService = inject(FileStoragesService);
  router = inject(Router);
  store = inject(Store);
  fb = new FormBuilder();

  form = this.fb.group({
    name: ['', [Validators.required]],
    fileMaxSize: [10_240, [Validators.required, Validators.min(1)]],
    compressionEnabled: [true],
    imgResizeMaxSize: [1024],
    imgDefaultFormat: ['webp'],
    quotaSize: [1_048_576, [Validators.required, Validators.min(1)]],
    allowedFileTypes: ['*'],
    isActive: [true],
  });

  isLoading = signal(false);

  constructor() {
    this.form.get('compressionEnabled')?.valueChanges.subscribe((enabled) => {
      this.updateImageValidators(enabled ?? true);
    });

    this.updateImageValidators(true);
  }

  updateImageValidators(compressionEnabled: boolean) {
    const imgResizeMaxSize = this.form.get('imgResizeMaxSize');
    const imgDefaultFormat = this.form.get('imgDefaultFormat');

    if (compressionEnabled) {
      imgResizeMaxSize?.setValidators([Validators.required, Validators.min(1)]);
      imgDefaultFormat?.setValidators([Validators.required]);
      imgResizeMaxSize?.enable();
      imgDefaultFormat?.enable();
    } else {
      imgResizeMaxSize?.clearValidators();
      imgDefaultFormat?.clearValidators();
      imgResizeMaxSize?.disable();
      imgDefaultFormat?.disable();
    }

    imgResizeMaxSize?.updateValueAndValidity();
    imgDefaultFormat?.updateValueAndValidity();
  }

  onSubmit() {
    if (this.form.valid) {
      this.store.dispatch(setLoading({ state: true }));
      this.isLoading.set(true);

      const val = this.form.getRawValue();
      
      const payload: any = {
        name: val.name,
        fileMaxSize: val.fileMaxSize,
        compressionEnabled: val.compressionEnabled,
        allowedFileTypes: val.allowedFileTypes || null,
        isActive: val.isActive,
        quotaSize: val.quotaSize,
      };

      if (val.compressionEnabled) {
        payload.imgResizeMaxSize = val.imgResizeMaxSize;
        payload.imgDefaultFormat = val.imgDefaultFormat;
      } else {
        payload.imgResizeMaxSize = null;
        payload.imgDefaultFormat = null;
      }
      
      this.fileStoragesService
        .createFileStorage(payload)
        .subscribe({
          next: (res: any) => {
            this.store.dispatch(setLoading({ state: false }));
            this.isLoading.set(false);
            this.router.navigate(['/file-storages', res.data.id]);
            new Snackbar('File storage created successfully', {
              iconSrc: '/success.png',
              position: 'bottom-right',
            });
          },
          error: (err) => {
            console.error('Error creating file storage:', err);
            this.store.dispatch(setLoading({ state: false }));
            this.isLoading.set(false);
            new Snackbar(err.error?.message || `Failed to create file storage`, {
              iconSrc: '/error.png',
              position: 'bottom-right',
            });
          }
        });
    } else {
      this.form.markAllAsTouched();
    }
  }

  isInvalid(controlName: string): boolean {
    const control = this.form.get(controlName);
    return !!(control && control.invalid && control.touched);
  }
}