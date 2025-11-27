import { AsyncPipe, DatePipe } from '@angular/common';
import { Component, inject, signal } from '@angular/core';
import { ActivatedRoute, NavigationEnd, Router, RouterLink, RouterLinkActive, RouterOutlet } from '@angular/router';
import { ConfirmDialog } from '@core/components/confirm-dialog/confirm-dialog';
import { FileStoragesService } from '@core/services/file-storages';
import { addParamHeader } from '@core/utils/add-param-header';
import { DialogService } from '@ngneat/dialog';
import { Store } from '@ngrx/store';
import { setLoading } from 'app/state/loading/loading.actions';
import Snackbar from 'awesome-snackbar';
import { catchError, tap } from 'rxjs';

@Component({
  selector: 'app-details',
  imports: [RouterLink, RouterLinkActive, AsyncPipe, DatePipe, RouterOutlet],
  templateUrl: './details.html',
  styleUrl: './details.css',
})
export class Details {
  route = inject(ActivatedRoute);
  router = inject(Router);
  fileStoragesService = inject(FileStoragesService);
  dialog = inject(DialogService)
  store = inject(Store)

  fileStorageId = this.route.snapshot.paramMap.get('id')!;

  copied = signal(false)

  fileStorage$ = this.fileStoragesService.getFileStorageById(this.fileStorageId).pipe(
    addParamHeader(':storageId', 'data.name'),
    catchError((err: any) => {
      if (err.status === 404) {
        this.router.navigate(['/404']);
      }
      new Snackbar(err.error?.message || `Failed to load file storage details`, {
        iconSrc: '/error.png',
        position: 'bottom-right',
      });
      this.router.navigate(['/file-storages']);
      return [];
    })
  )

  onActivate(storage: any) {
    history.replaceState(
      { ...history.state, ':storageId': storage.data.name },
      ''
    );
    (this.router.events as any).next(new NavigationEnd(0, this.router.url, this.router.url));
  }

  onDeleteFileStorage() {
    let d = this.dialog.open(ConfirmDialog, {
      data: {
        title: 'Confirm Deletion',
        content: 'Are you sure you want to delete this file storage?'
      }
    });
    d.afterClosed$.subscribe((result: any) => {
      if (result?.confirm) {
        this.store.dispatch(setLoading({ state: true }));
        this.fileStoragesService.deleteFileStorage(this.fileStorageId).subscribe({
          next: () => {
            this.store.dispatch(setLoading({ state: false }));
            this.router.navigate(['/file-storages']);
          },
          error: (err) => {
            this.store.dispatch(setLoading({ state: false }));
            new Snackbar(err.error?.message || `Failed to delete file storage`, {
              iconSrc: '/error.png',
              position: 'bottom-right',
            });
            console.error(err);
          }
        });
      }
    });
  }
}
