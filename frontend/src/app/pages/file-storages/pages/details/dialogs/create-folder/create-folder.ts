import { Component, inject } from '@angular/core';
import { FormBuilder, ReactiveFormsModule } from '@angular/forms';
import { DialogRef } from '@ngneat/dialog';

@Component({
  selector: 'app-create-folder',
  imports: [ReactiveFormsModule],
  templateUrl: './create-folder.html',
  styleUrl: './create-folder.css',
})
export class CreateFolder {
  dialogRef = inject(DialogRef);
  
  fb = new FormBuilder();
  form = this.fb.group({
    name: [''],
  });

  onSubmit() {
    if (this.form.valid) {
      this.dialogRef.close({ 
        status: 'create',
        name: this.form.value.name 
      });
    }
  }

  onCancel() {
    this.dialogRef.close({ status: 'cancel' });
  }
}