import { Component, inject } from '@angular/core';
import { DialogRef } from '@ngneat/dialog';

@Component({
  selector: 'app-confirm-dialog',
  imports: [],
  templateUrl: './confirm-dialog.html',
  styleUrl: './confirm-dialog.scss',
})
export class ConfirmDialog {
  dialogRef = inject(DialogRef);
  data = this.dialogRef.data;

  handleConfirm() {
    this.dialogRef.close({ confirm: true });
  }

  handleCancel() {
    this.dialogRef.close({ confirm: false });
  }
}
