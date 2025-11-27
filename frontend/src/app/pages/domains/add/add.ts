import { Component, inject, signal, effect } from '@angular/core';
import { FormBuilder, ReactiveFormsModule, Validators } from '@angular/forms';
import { Router } from '@angular/router';
import { DomainsService } from '@core/services/domains';
import { Store } from '@ngrx/store';
import { setLoading } from 'app/state/loading/loading.actions';
import { CommonModule } from '@angular/common';
import Snackbar from 'awesome-snackbar';
import { UnitConvertPipe } from '@core/pipes/unit-convert-pipe';

@Component({
  selector: 'app-add',
  imports: [ReactiveFormsModule, UnitConvertPipe],
  templateUrl: './add.html',
  styleUrl: './add.css',
})
export class Add {
  domainsService = inject(DomainsService);
  router = inject(Router);
  store = inject(Store);
  fb = new FormBuilder();

  domainTypes = [
    { value: 'reverse_proxy', label: 'Reverse Proxy' },
    { value: 'web_server', label: 'Web Server' },
    { value: 'static_host', label: 'Static Host' },
    { value: 'custom', label: 'Custom Config' },
  ];

  form = this.fb.group({
    domain: ['', [Validators.required]],
    domainType: ['reverse_proxy', [Validators.required]],
    
    nginxTargetHost: ['127.0.0.1:3000'],
    nginxRootPath: ['/var/www/html'],
    nginxConfigContent: [''],

    clientMaxBodySize: [52428800, [Validators.required, Validators.min(1)]],
    isSsl: [true],
    isActive: [true],
    sslCertificatePath: [''],
    sslCertificateKeyPath: [''],
  });

  isLoading = signal(false);

  constructor() {
    this.form.get('domainType')?.valueChanges.subscribe((type) => {
      this.updateValidators(type || 'reverse_proxy');
    });

    this.updateValidators('reverse_proxy');
  }

  updateValidators(type: string) {
    const targetHost = this.form.get('nginxTargetHost');
    const rootPath = this.form.get('nginxRootPath');
    const configContent = this.form.get('nginxConfigContent');
    const maxBody = this.form.get('clientMaxBodySize');
    const isSsl = this.form.get('isSsl');

    // Default: Disable specific inputs
    targetHost?.disable();
    rootPath?.disable();
    configContent?.disable();
    
    // Default: Enable common inputs
    maxBody?.enable();
    isSsl?.enable();

    if (type === 'reverse_proxy') {
      targetHost?.enable();
      targetHost?.setValidators([Validators.required]);
    } else if (type === 'web_server' || type === 'static_host') {
      rootPath?.enable();
      rootPath?.setValidators([Validators.required]);
    } else if (type === 'custom') {
      configContent?.enable();
      configContent?.setValidators([Validators.required]);
      
      // Disable common inputs for Custom type
      maxBody?.disable();
      isSsl?.disable();
      isSsl?.setValue(false); // Clean state
    }
  }

  onSubmit() {
    if (this.form.valid) {
      this.store.dispatch(setLoading({ state: true }));
      this.isLoading.set(true);

      const val = this.form.value;

      const payload = {
        domain: val.domain ?? '',
        domainType: val.domainType ?? 'reverse_proxy',
        
        nginxTargetHost: val.nginxTargetHost || undefined,
        nginxRootPath: val.nginxRootPath || undefined,
        nginxConfigContent: val.nginxConfigContent || undefined,

        clientMaxBodySize: val.clientMaxBodySize ?? 52428800,
        isSsl: val.isSsl ?? true,
        isActive: val.isActive ?? true,
        
        sslCertificatePath: val.sslCertificatePath || undefined,
        sslCertificateKeyPath: val.sslCertificateKeyPath || undefined
      };

      this.domainsService.createDomain(payload).subscribe({
        next: (res: any) => {
          this.isLoading.set(false);
          this.store.dispatch(setLoading({ state: false }));
          this.router.navigate(['/domains/', res.data.id]);
          new Snackbar(`Domain created successfully`, {
            iconSrc: '/success.png',
            position: 'bottom-right',
          });
        },
        error: (err) => {
          this.isLoading.set(false);
          this.store.dispatch(setLoading({ state: false }));
          new Snackbar(err.error?.message || `Failed to create domain`, {
            iconSrc: '/error.png',
            position: 'bottom-right',
          });
          console.error(err);
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