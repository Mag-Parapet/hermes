import { AsyncPipe } from '@angular/common';
import { Component, inject, signal } from '@angular/core';
import { FormBuilder, ReactiveFormsModule, Validators } from '@angular/forms';
import { ActivatedRoute, Router } from '@angular/router';
import { FileStoragesService } from '@core/services/file-storages';
import { addParamHeader } from '@core/utils/add-param-header';
import { Store } from '@ngrx/store';
import { setLoading } from 'app/state/loading/loading.actions';
import Snackbar from 'awesome-snackbar';
import { catchError, tap } from 'rxjs';

@Component({
  selector: 'app-edit',
  imports: [ReactiveFormsModule, AsyncPipe],
  templateUrl: './edit.html',
  styleUrl: './edit.css',
})
export class Edit {
  fileStoragesService = inject(FileStoragesService);
  router = inject(Router);
  route = inject(ActivatedRoute);
  store = inject(Store);
  fb = new FormBuilder();

  storageId = this.route.snapshot.paramMap.get('id')!;

  storage$ = this.fileStoragesService.getFileStorageById(this.storageId).pipe(
    tap((res: any) => {
      this.store.dispatch(setLoading({ state: false }));

      this.form.patchValue({
        name: res.data.name,
        fileMaxSize: res.data.fileMaxSize,
        compressionEnabled: res.data.compressionEnabled,
        imgResizeMaxSize: res.data.imgResizeMaxSize,
        imgDefaultFormat: res.data.imgDefaultFormat,
        quotaSize: res.data.quotaSize,
        allowedFileTypes: res.data.allowedFileTypes,
        isActive: res.data.isActive,
      });

      this.updateImageValidators(res.data.compressionEnabled);
      this.isLoading.set(false);
    }),
    addParamHeader(':storageId', 'data.name'),
    catchError((err: any) => {
      if (err.status === 404) {
        this.router.navigate(['/404']);
      }
      new Snackbar(err.error?.message || `Failed to load file storage`, {
        iconSrc: '/error.png',
        position: 'bottom-right',
      });
      this.router.navigate(['/file-storages']);
      return [];
    })
  );
    

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
        .updateFileStorage(this.storageId, payload)
        .subscribe({
          next: (res: any) => {
            this.store.dispatch(setLoading({ state: false }));
            this.isLoading.set(false);
            this.router.navigate(['/file-storages', res.data.id]);
          },
          error: (err) => {
            console.error('Error creating file storage:', err);
            this.store.dispatch(setLoading({ state: false }));
            this.isLoading.set(false);
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
