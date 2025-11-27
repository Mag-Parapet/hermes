import { HttpClient } from '@angular/common/http';
import { inject, Injectable } from '@angular/core';
import { environment } from 'environments/environment';

@Injectable({
  providedIn: 'root',
})
export class FileStoragesService {
  private http = inject(HttpClient);
  private API_URL = environment.API_URL + 'storages';

  getAllFileStorages(page: number, pageSize: number, search?: string) {
    const params = new URLSearchParams();
    params.append('page', page.toString());
    params.append('pageSize', pageSize.toString());
    if (search) {
      params.append('search', search);
    }
    return this.http.get<any[]>(`${this.API_URL}?${params.toString()}`);
  }
  
  getFileStorageById(id: string) {
    return this.http.get<any>(`${this.API_URL}/${id}`);
  }

  createFileStorage(data: any) {
    return this.http.post<any>(`${this.API_URL}`, data);
  }

  updateFileStorage(id: string, data: any) {
    return this.http.put<any>(`${this.API_URL}/${id}`, data);
  }

  deleteFileStorage(id: string) {
    return this.http.delete<any>(`${this.API_URL}/${id}`);
  }

  upload(storageId: string, formData: FormData) {
    return this.http.post<any>(`${this.API_URL}/${storageId}/upload`, formData);
  }

  getFiles(storageId: string, page: number, pageSize: number, path: string[] = [], search?: string) {
    const params = new URLSearchParams();
    params.append('page', page.toString());
    params.append('pageSize', pageSize.toString());
    if (path.length > 0) {
      params.append('path', '/' + path.join('/'));
    } else {
      params.append('path', '/');
    }
    if (search) {
      console.log('Appending search param:', search);
      params.append('search', search);
    }
    return this.http.get<any[]>(`${this.API_URL}/${storageId}/files?${params.toString()}`);
  }

  createFolder(storageId: string, payload: any) {
    return this.http.post<any>(`${this.API_URL}/${storageId}/folders`, payload);
  }

  deleteFile(storageId: string, fileId: string) {
    return this.http.delete<any>(`${this.API_URL}/${storageId}/files/${fileId}`);
  }
}
