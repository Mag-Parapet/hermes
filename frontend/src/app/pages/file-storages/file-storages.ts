import { AsyncPipe, DatePipe } from '@angular/common';
import { Component, inject, signal } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { FormBuilder, ReactiveFormsModule } from '@angular/forms';
import { RouterLink } from '@angular/router';
import { ContentTable } from '@core/components/content-table/content-table';
import { FileStoragesService } from '@core/services/file-storages';
import { Store } from '@ngrx/store';
import { setLoading } from 'app/state/loading/loading.actions';
import { map } from 'rxjs';

@Component({
  selector: 'app-file-storages',
  imports: [ContentTable, RouterLink, AsyncPipe, ReactiveFormsModule],
  templateUrl: './file-storages.html',
  styleUrl: './file-storages.css',
  providers: [DatePipe]
})
export class FileStorages {
  fileStoragesService = inject(FileStoragesService);
  datePipe = inject(DatePipe);
  store = inject(Store);

  fb = new FormBuilder();
  form = this.fb.group({
    search: [''],
  });

  columns = ['Name', 'Max File Size', 'Compress', 'Active', 'Last Updated'];

  data = [];

  page = signal<number>(1);
  pageSize = signal<number>(10);

  fileStorages$: any

  constructor() {
    this.setFileStorages()
    this.form.valueChanges.pipe(takeUntilDestroyed()).subscribe(() => {
      this.page.set(1);
      this.setFileStorages();
    })
  }

  onPageChange(page: number) {
    this.page.set(page);
    this.setFileStorages();
  }

  onPageSizeChange(size: number) {
    this.pageSize.set(size);
    this.setFileStorages();
  }


  setFileStorages() {
    const { search } = this.form.value;
    this.store.dispatch(setLoading({ state: true }));
    this.fileStorages$ = this.fileStoragesService.getAllFileStorages(
      this.page(),
      this.pageSize(),
      search!
    ).pipe(
      map((res: any) => {
        this.store.dispatch(setLoading({ state: false }));
        return {
          rows: res.data.fileStorages.map((fs: any) => [
            fs.name,
            fs.maxFileSize + ' KB',
            fs.compress ? 'Yes' : 'No',
            fs.isActive ? 'Yes' : 'No',
            this.datePipe.transform(fs.updatedAt, 'medium')
          ]),
          total: res.data.total
        };
      })
    );
  }
}
