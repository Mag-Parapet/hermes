import { Component, computed, inject, input, Signal, signal, WritableSignal } from '@angular/core';
import { Path } from '../../ui/path/path';
import { FileStoragesService } from '@core/services/file-storages';
import { ActivatedRoute } from '@angular/router';
import { AsyncPipe, DatePipe, SlicePipe } from '@angular/common';
import { FormBuilder, FormsModule, ReactiveFormsModule } from '@angular/forms';
import { DialogService } from '@ngneat/dialog';
import { UploadFiles } from '../../dialogs/upload-files/upload-files';
import { CreateFolder } from '../../dialogs/create-folder/create-folder';
import { ConfirmDialog } from '@core/components/confirm-dialog/confirm-dialog';
import { TruncatePipe } from '@core/pipes/truncate-pipe';
import { Store } from '@ngrx/store';
import { setLoading } from 'app/state/loading/loading.actions';
import { debounceTime, of } from 'rxjs';
import Snackbar from 'awesome-snackbar';
import { UnitConvertPipe } from '@core/pipes/unit-convert-pipe';
import { FileDetails } from '../../dialogs/file-details/file-details';
import { ImagePreview } from '../../dialogs/image-preview/image-preview';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';

@Component({
  selector: 'app-files',
  imports: [Path, ReactiveFormsModule, FormsModule, DatePipe, TruncatePipe, UnitConvertPipe],
  templateUrl: './files.html',
  styleUrl: './files.css'
})
export class Files {
  fileStoragesService = inject(FileStoragesService)
  route = inject(ActivatedRoute)
  dialog = inject(DialogService)
  store = inject(Store)

  fileStorageId = this.route.snapshot.parent!.paramMap.get('id')!;

  data: any = signal({}); 

  pagesRange: Signal<number[]> = computed(() => {
    const totalPages = this.data()?.pagination?.totalPages || 1;
    return Array.from({ length: totalPages }, (_, i) => i + 1);
  });

  page: WritableSignal<number> = signal(1);
  pageSize: WritableSignal<number> = signal(10)
  currentPath: WritableSignal<string[]> = signal([]);

  fb = new FormBuilder()
  form = this.fb.group({
    search: [''],
  });

  constructor() {
    this.setFiles();
    this.form.get('search')!.valueChanges.pipe(debounceTime(300), takeUntilDestroyed()).subscribe(() => {
      this.currentPath.set([]);
      this.page.set(1);
      this.setFiles();
    });
  }

  onEnterFolder(folderName: string) {
    const newPath = [...this.currentPath(), folderName];
    this.currentPath.set(newPath);
    this.setFiles(newPath);
  }
  
  onPathChange(newPath: string[]) {
    this.currentPath.set(newPath);
    this.setFiles(newPath);
  }

  onChangePage(page: string) {
    this.page.set(Number(page));
    this.setFiles();
  }

  onChangePageSize(size: string) {
    this.pageSize.set(Number(size));
    this.setFiles();
  }

  onPreviousPage() {
    if (this.page() > 1) {
      this.page.set(this.page() - 1);
      this.setFiles();
    }
  }

  onOpenFileDetails(file: any) {
    this.dialog.open(FileDetails, { data: { file } });
  }

  allowFilePreview(file: any): boolean {
    return ['image/webp', 'image/png', 'image/jpg'].includes(file.fileType);
  }

  onOpenFilePreview(file: any) {
    if (['image/webp', 'image/png', 'image/jpg'].includes(file.fileType)) {
      this.dialog.open(ImagePreview, { data: { file } });
    }
  }

  onDeleteFile(file: any) {
    let d = this.dialog.open(ConfirmDialog, {
      data: {
        title: 'Confirm Deletion',
        content: `Are you sure you want to delete the ${file.fileType === 'folder' ? 'folder' : 'file'} "${file.name}"?`
      }
    });
    d.afterClosed$.subscribe((result: any) => {
      if (result?.confirm) {
        this.store.dispatch(setLoading({ state: true }));
        this.fileStoragesService.deleteFile(this.fileStorageId, file.id).subscribe({
          next: (response: any) => {
            this.setFiles(this.currentPath());
            this.store.dispatch(setLoading({ state: false }));
            new Snackbar(`${file.fileType === 'folder' ? 'Folder' : 'File'} "${file.name}" deleted successfully`, {
              iconSrc: '/success.png',
              position: 'bottom-right',
            });
          },
          error: (err) => {
            this.store.dispatch(setLoading({ state: false }));
            new Snackbar(err.error?.message || `Failed to delete ${file.fileType === 'folder' ? 'folder' : 'file'} "${file.name}"`, {
              iconSrc: '/error.png',
              position: 'bottom-right',
            });
            console.error(err);
          }
        });
      }
    });
  }

  onNextPage() {
    const totalPages = this.data()?.pagination?.totalPages || 1;
    if (this.page() < totalPages) {
      this.page.set(this.page() + 1);
      this.setFiles();
    }
  }

  onUploadFiles() {
    let d = this.dialog.open(UploadFiles);
    d.afterClosed$.subscribe((result: any) => {
      if (result?.status === 'files') {
        let formData = new FormData();
        formData.append('path', this.currentPath().length ? this.currentPath().join('/') : '/');
        for (let i = 0; i < result.files.length; i++) {
          formData.append('files', result.files[i]);
        }
        this.store.dispatch(setLoading({ state: true }));
        this.fileStoragesService.upload(this.fileStorageId, formData).subscribe({
          next: (response: any) => {
            this.setFiles(this.currentPath());
            this.store.dispatch(setLoading({ state: false }));
            new Snackbar('Files uploaded successfully', {
              iconSrc: '/success.png',
              position: 'bottom-right',
            });
          },
          error: (err) => {
            this.store.dispatch(setLoading({ state: false }));
            new Snackbar(err.error?.message || `Failed to upload files`, {
              iconSrc: '/error.png',
              position: 'bottom-right',
            });
            console.error(err);
          }
        })
      }
    });
  }

  onCreateFolder() {
    let d = this.dialog.open(CreateFolder);
    d.afterClosed$.subscribe((data: any) => {
      if (data?.status === 'create') {
        let payload = {
          name: data.name,
          path: this.currentPath().length ? '/' + this.currentPath().join('/') : '/'
        }
        this.store.dispatch(setLoading({ state: true }));
        this.fileStoragesService.createFolder(this.fileStorageId, payload).subscribe({
          next: (response: any) => {
            this.setFiles(this.currentPath());
            this.store.dispatch(setLoading({ state: false }));
            new Snackbar('Folder created successfully', {
              iconSrc: '/success.png',
              position: 'bottom-right',
            });
          },
          error: (err) => {
            this.store.dispatch(setLoading({ state: false }));
            new Snackbar(err.error?.message || `Failed to create folder`, {
              iconSrc: '/error.png',
              position: 'bottom-right',
            });
            console.error(err);
          }
        })
      }
    });
  }

  setFiles(path: string[] = []) {
    this.store.dispatch(setLoading({ state: true }));
    this.fileStoragesService.getFiles(this.fileStorageId, this.page(), this.pageSize(), this.currentPath(), this.form.value.search!).subscribe({
      next: (response: any) => {
        this.data.set(response.data);
        this.store.dispatch(setLoading({ state: false }));
      },
      error: (err) => {
        this.data.set({ files: [], pagination: {} });
        new Snackbar(err.error?.message || `Failed to load files`, {
          iconSrc: '/error.png',
          position: 'bottom-right',
        });
        this.store.dispatch(setLoading({ state: false }));
        console.error(err);
      }
    })
  }
}
